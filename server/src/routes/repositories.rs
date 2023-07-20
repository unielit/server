use crate::{
    errors::AppError,
    models::{repositories, repositories::NewRepository, Result},
    routes::success,
    services::github,
    DbPool,
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};

use super::parse_auth_token;

#[derive(Deserialize, Debug)]
struct InputRepository {
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
    pub params: serde_json::Value,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/repos").service(web::resource("").route(web::post().to(create_repo))), // .service(web::resource("/{id}").route(web::get().to(get_project)))
                                                                                            // .service(web::resource("/{id}").route(web::patch().to(update_project)))
                                                                                            // .service(web::resource("/find/{name}").route(web::get().to(find_project)))
                                                                                            // .service(web::resource("/find/all").route(web::get().to(get_user_projects)))
    );
}

async fn create_repo(
    input: web::Json<InputRepository>,
    pool1: web::Data<DbPool>,
    pool2: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;
    let input = input.into_inner();
    let repo_owner = repositories::RepositoryOwner {
        name: input.name.clone(),
        owner: input.owner.clone(),
        is_organization: input.is_organization,
    };

    web::block(move || {
        let mut conn = pool1.get()?;

        if repositories::is_repo_exist(&mut conn, repo_owner)? {
            return Err(AppError::RecordAlreadyExists);
        }

        Ok(())
    })
    .await?
    .map_err(AppError::from)?;

    let api = github::GitHubAPI::new(&token)?;
    let api_response = match input.is_organization {
        true => api.create_org_repo(&input.owner, input.params).await,
        false => api.create_personal_repo(input.params).await,
    }?;

    if !api_response.status().is_success() && !api_response.status().is_informational() {
        let response = HttpResponse::build(api_response.status())
            .content_type("text/plain")
            .body(
                api_response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Github API request failed. Error: {}", e)),
            );

        return Ok(response);
    }

    let json: serde_json::Value = api_response.json::<serde_json::Value>().await?;

    web::block(move || {
        let mut conn = pool2.get()?;
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
