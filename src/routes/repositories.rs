use crate::{
    errors::AppError,
    models::{
        repositories::{self, NewRepository, RepositoryKey, RepositoryOwner, UpdateRepository},
        Result,
    },
    routes::success,
    services::github::GitHubAPI,
    DbPool,
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use utoipa::ToSchema;
use uuid::Uuid;

use super::parse_auth_token;

const DESIGN_FILE_NAME: &str = "design.json";

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputRepository {
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
    pub params: serde_json::Value,
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveRepoDesign {
    pub message: String,
    pub content: serde_json::Value,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/repos")
            .service(web::resource("").route(web::post().to(create_repo)))
            .service(web::resource("/{id}/save_design").route(web::put().to(save_repo_design))),
    );
}

/// Create a repository
///
/// A User Bearer access token should be provided to create a record.
/// The access token provided must be associated with a user account.
///
/// Unique constraint for repository record consists of (name && owner && is_organization).
///
/// If during creation request there is no such repository in our database by (name && owner && is_organization)
/// then server sends request to Github API to create the repository.
///
/// If the repository is successfully created on Github, it will be created in our database.
///
/// IMPORTANT:
/// - It is possible that a repository exists in our database but cannot be created on Github.
/// This happens if the repository hasn't been deleted by our API, but the repository has been deleted on Github.
/// In this case it is not possible to create a repository using our API until specific record is not deleted.
/// - It is possible that a repository has been created on Github but no record has been created in our database.
/// In this case the Github repo should be deleted and the creation request repeated.
///
/// In the future, functionality will be added to update the remote repository record in our local repository record.
#[utoipa::path(
    post,
    context_path = "/repos",
    path = "",
    tag = "Repositories",
    request_body(content = InputRepository, description = "Input Repository in JSON format. Provide all needed parameters in JSON format for creating repository in Github. Parameters description could be found here. https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user", content_type = "application/json"),
    responses(
        (status = OK, body = Repository),
        (status = BAD_REQUEST, description = "Unique constaint violation."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token."),
        (status = 502, body = String, content_type = "text/plain", description = "Github API request failed.")
    ),
    security(
        ("http" = [])
    )
)]
async fn create_repo(
    input: web::Json<InputRepository>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;
    let input: InputRepository = input.into_inner();

    let api_response = create_github_repo(&token, input.to_owned(), pool.clone()).await?;

    if !api_response.status().is_success() && !api_response.status().is_informational() {
        let response = HttpResponse::BadGateway().content_type("text/plain").body(
            api_response
                .text()
                .await
                .unwrap_or_else(|e| AppError::GithubAPIError(e.to_string()).to_string()),
        );

        return Ok(response);
    }

    let json: serde_json::Value = api_response.json::<serde_json::Value>().await?;

    web::block(move || {
        let mut conn = pool.get()?;
        let mut html_url: String = format!(
            "https://github.com/{ow}/{nm}",
            ow = input.owner,
            nm = input.name
        );

        if let Some(json_url) = json.get("html_url").cloned() {
            if let Ok(url) = serde_json::from_value(json_url) {
                html_url = url;
            }
        }

        let new_repo = NewRepository {
            name: &input.name,
            owner: &input.owner,
            is_organization: input.is_organization,
            design_file_sha: None,
            html_url: &html_url,
        };

        repositories::create_repo(&mut conn, new_repo)
    })
    .await?
    .map(success)
}

async fn create_github_repo(
    token: &str,
    input: InputRepository,
    pool: web::Data<DbPool>,
) -> Result<reqwest::Response> {
    let block_input = input.to_owned();

    web::block(move || {
        let mut conn = pool.get()?;
        let repo_owner = repositories::RepositoryOwner {
            name: block_input.name,
            owner: block_input.owner,
            is_organization: input.is_organization,
        };

        if repositories::is_repo_exist(&mut conn, repo_owner)? {
            return Err(AppError::RecordAlreadyExists);
        }

        Ok(())
    })
    .await?
    .map_err(AppError::from)?;

    let api = GitHubAPI::new()?;
    let api_response = match input.is_organization {
        true => {
            api.create_org_repo(token, &input.owner, input.params.clone())
                .await
        }
        false => api.create_personal_repo(token, input.params.clone()).await,
    }?;

    Ok(api_response)
}

/// Save design to repository
///
/// A User Bearer access token should be provided to create a record.
/// The access token provided must be associated with a user account.
///
#[utoipa::path(
    put,
    context_path = "/repos",
    path = "/{id}/save_design",
    tag = "Repositories",
    request_body(content = SaveRepoDesign, content_type = "application/json"),
    responses(
        (status = OK),
        (status = BAD_REQUEST, description = "Repo is not found."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token."),
        (status = 502, body = String, content_type = "text/plain", description = "Github API request failed.")
    ),
    security(
        ("http" = [])
    )
)]
async fn save_repo_design<'a>(
    repo_id: web::Path<Uuid>,
    info: web::Json<SaveRepoDesign>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;
    let pool1 = pool.to_owned();

    let repo = web::block(move || {
        let mut conn = pool1.get()?;

        repositories::find_repo(&mut conn, RepositoryKey::ID(repo_id.into_inner()))
    })
    .await?
    .map_err(AppError::from)?;

    let api = GitHubAPI::new()?;
    let info: SaveRepoDesign = info.into_inner();
    let content = info.content;
    let content_data = serde_json::to_vec(&content).map_err(AppError::from)?;
    let base64_content = general_purpose::STANDARD_NO_PAD.encode(content_data);

    let file_commit = api
        .save_file_content(
            &token,
            RepositoryOwner {
                name: repo.name,
                owner: repo.owner,
                is_organization: repo.is_organization,
            },
            DESIGN_FILE_NAME,
            &info.message,
            &base64_content,
            None,
        )
        .await?;

    web::block(move || {
        let mut conn = pool.get()?;

        repositories::update_repo(
            &mut conn,
            repo.id,
            UpdateRepository {
                design_file_sha: file_commit.content.map(|c| c.sha).as_deref(),
                ..Default::default()
            },
        )
    })
    .await?
    .map_err(AppError::from)?;

    Ok(HttpResponse::Ok())
}
