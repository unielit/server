use crate::{
    errors::AppError,
    models::{users, Result},
    routes::success,
    services::github::GitHubAPI,
    DbPool,
};
use actix_web::{web, Responder};
use reqwest::*;
use serde_json::json;
use std::env;

struct GitHubAuth {
    client: Client,
    base_url: Url,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AccessTokenQuery {
    Code { code: String },
    AccessToken { access_token: String },
}

#[derive(Serialize, Default, Debug)]
struct GenerateAccessTokenParams {
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: Option<String>,
    repository_id: Option<String>,
}

#[derive(Serialize, Default, Debug)]
struct RefreshAccessTokenParams {
    client_id: String,
    client_secret: String,
    grant_type: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GenerateAccessTokenResponse {
    access_token: String,
    expires_in: i32,
    refresh_token: String,
    refresh_token_expires_in: i32,
    scope: String,
    token_type: String,
}

impl GitHubAuth {
    pub fn new() -> Result<Self> {
        let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be present.");
        let client_secret =
            env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be present.");
        let base_url = Url::parse("https://github.com")?;
        let client = reqwest::Client::builder().build()?;

        Ok(GitHubAuth {
            client,
            base_url,
            client_id,
            client_secret,
        })
    }

    pub async fn generate_access_token(
        &self,
        params: GenerateAccessTokenParams,
    ) -> Result<GenerateAccessTokenResponse> {
        let mut url = self.base_url.clone();
        url.set_path("/login/oauth/access_token");

        let response = self
            .client
            .post(url)
            .query(&params)
            .send()
            .await
            .map_err(AppError::from)?;

        let body = response.text().await.map_err(AppError::from)?;

        if let Ok(success) = serde_urlencoded::from_str::<GenerateAccessTokenResponse>(&body) {
            return Ok(success);
        }

        Err(AppError::GithubAuthError(body))
    }

    pub async fn refresh_access_token(
        &self,
        params: RefreshAccessTokenParams,
    ) -> Result<GenerateAccessTokenResponse> {
        let mut url = self.base_url.clone();
        url.set_path("/login/oauth/access_token");

        let response = self
            .client
            .post(url)
            .query(&params)
            .send()
            .await
            .map_err(AppError::from)?;

        let body = response.text().await.map_err(AppError::from)?;

        if let Ok(success) = serde_urlencoded::from_str::<GenerateAccessTokenResponse>(&body) {
            return Ok(success);
        }

        Err(AppError::GithubAuthError(body))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth/github")
            .service(web::resource("/access_token").route(web::post().to(generate_access_token))),
    );
}

/// Generate user access token
/// 
/// IMPORTANT!!!
/// 
/// ONLY one query parameter should be provided.
/// Either code to generate new tokens or access token
/// to refresh existing tokens.
#[utoipa::path(
    post,
    context_path = "/auth/github",
    path = "/access_token",
    tag = "Auth Github",
    params(
        (
            "code" = Option<String>,
            Query,
            description = "Code to request tokens.",
        ),
        (
            "access_token" = Option<String>,
            Query,
            description = "Old Access Token to refresh tokens.",
        ),
    ),
    responses(
        (
            status = OK, 
            description = "Return a new access token and expiration time in seconds for the user",
            content_type = "application/json",
            example = json!({
                "access_token" : "gnu_token",
                "expires_in" : 28800,
            })
        ),
        (status = BAD_REQUEST, description = "There is no user connected to provided access_token."),
        (status = 502, body = String, content_type = "text/plain", description = "Github AUTH API request failed.")
    ),
)]
async fn generate_access_token(
    query: web::Query<AccessTokenQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    let github_auth = GitHubAuth::new()?;
    let query = query.into_inner();

    let response = match query {
        AccessTokenQuery::Code { code } => github_auth
            .generate_access_token(GenerateAccessTokenParams {
                client_id: github_auth.client_id.clone(),
                client_secret: github_auth.client_secret.clone(),
                code,
                ..Default::default()
            })
            .await
            .map_err(AppError::from)?,
        AccessTokenQuery::AccessToken { access_token } => {
            let pool = pool.clone();
            let refresh_token = web::block(move || {
                let mut conn = pool.get()?;
                let roled_user = users::find_user(&mut conn, users::UserKey::Token(&access_token))?;

                users::get_user_refresh_token(&mut conn, roled_user.user.id)
            })
            .await??;

            github_auth
                .refresh_access_token(RefreshAccessTokenParams {
                    client_id: github_auth.client_id.clone(),
                    client_secret: github_auth.client_secret.clone(),
                    grant_type: "refresh_token".to_string(),
                    refresh_token,
                })
                .await
                .map_err(AppError::from)?
        }
    };

    save_access_token_response(response, pool).await
}

async fn save_access_token_response(
    response: GenerateAccessTokenResponse,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    let github_api = GitHubAPI::new()?;
    let user = github_api.get_auth_user(&response.access_token).await?;
    let primary_email = github_api
        .get_user_primary_email(&response.access_token)
        .await?;

    let new_user = users::NewUser {
        name: user.name.unwrap_or(user.login),
        email: primary_email.email.clone(),
        access_token: None,
    };
    let save_token = users::TokenData {
        access_token: response.access_token.clone(),
        expires_in: response.expires_in,
        refresh_token: response.refresh_token.clone(),
        refresh_token_expires_in: response.refresh_token_expires_in,
        scope: response.scope.clone(),
        token_type: response.token_type.clone(),
    };

    web::block(move || {
        let mut conn = pool.get()?;

        users::save_user_token_data(&mut conn, new_user, save_token)
    })
    .await??;

    Ok(success(json!({
        "access_token" : response.access_token,
        "expires_in" : response.expires_in,
    })))
}
