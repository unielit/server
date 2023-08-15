use crate::{errors::AppError, models::Result};
use actix_web::HttpRequest;
use actix_web::{dev::ServiceRequest, http::header, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;

pub(super) mod auth;
pub(super) mod designs;
pub(super) mod projects;
pub(super) mod repositories;
pub(super) mod users;

fn success<T>(res: T) -> HttpResponse
where
    T: serde::Serialize,
{
    HttpResponse::Ok().json(res)
}

pub fn parse_auth_token(req: HttpRequest) -> Result<String> {
    let bearer_token = req
    .headers()
    .get(header::AUTHORIZATION)
    .ok_or(AppError::AuthError)?
    .to_str()?
    .to_owned();

    let parts: Vec<&str> = bearer_token.split_whitespace().collect();
    if parts.len() == 2 && parts[0] == "Bearer" {
        return Ok(parts[1].to_string());
    }

    Err(AppError::AuthError)
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> core::result::Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req.app_data::<Config>().cloned().unwrap_or_default();

    if auth::validate_token(credentials.token())
        .await
        .ok()
        .unwrap_or(false)
    {
        return Ok(req);
    }

    Err((AuthenticationError::from(config).into(), req))
}
