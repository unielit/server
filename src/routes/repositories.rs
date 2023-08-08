use crate::{
    errors::AppError,
    models::{repositories, repositories::NewRepository, Result},
    routes::success,
    services::github,
    DbPool,
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use utoipa::ToSchema;

use super::parse_auth_token;

#[derive(Deserialize, ToSchema, Debug)]
pub struct InputRepository {
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
    pub params: serde_json::Value,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/repos").service(web::resource("").route(web::post().to(create_repo))), 
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
    let api_response = create_github_repo(&token, &input, pool.clone()).await?;

    if !api_response.status().is_success() && !api_response.status().is_informational() {
        let response = HttpResponse::BadGateway()
            .content_type("text/plain")
            .body(
                api_response
                    .text()
                    .await
                    .unwrap_or_else(|e| AppError::GithubAPIError(e.to_string()).to_string())
            );

        return Ok(response);
    }

    let json: serde_json::Value = api_response.json::<serde_json::Value>().await?;

    web::block(move || {
        let mut conn = pool.get()?;
        let mut html_url = format!("https://github.com/{ow}/{nm}", ow = input.owner, nm = input.name);

        if let Some(json_url) = json.get("html_url") {
            if let Ok(url) = serde_json::from_value(json_url.clone()) {
                html_url = url
            }
        }

        repositories::create_repo(
            &mut conn,
            NewRepository {
                name: input.name,
                owner: input.owner,
                is_organization: input.is_organization,
                html_url,
            },
        )
    })
    .await?
    .map(success)
}

async fn create_github_repo(token: &str, input: &InputRepository, pool: web::Data<DbPool>) -> Result<reqwest::Response> {
    let repo_owner = repositories::RepositoryOwner {
        name: input.name.clone(),
        owner: input.owner.clone(),
        is_organization: input.is_organization,
    };
    
    web::block(move || {
        let mut conn = pool.get()?;

        if repositories::is_repo_exist(&mut conn, repo_owner)? {
            return Err(AppError::RecordAlreadyExists);
        }

        Ok(())
    })
    .await?
    .map_err(AppError::from)?;

    let api = github::GitHubAPI::new()?;
    let api_response = match input.is_organization {
        true => api.create_org_repo(&token, &input.owner, input.params.clone()).await,
        false => api.create_personal_repo(&token, input.params.clone()).await,
    }?;

    Ok(api_response)
}