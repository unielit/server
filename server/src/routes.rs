use crate::{auth, errors::AppError, models::Result};
use actix_web::HttpRequest;
use actix_web::{dev::ServiceRequest, Error, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;

pub(super) mod projects;
pub(super) mod users;

fn success<T>(res: T) -> impl Responder
where
    T: serde::Serialize,
{
    HttpResponse::Ok().json(res)
}

pub fn parse_auth_token(req: HttpRequest) -> Result<String> {
    Ok(req
        .headers()
        .get("Authorization")
        .ok_or(AppError::AuthError)?
        .to_str()?
        .to_owned())
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> core::result::Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_default();

    if auth::validate_token(credentials.token())
        .await
        .ok()
        .unwrap_or(false)
    {
        return Ok(req);
    }

    Err((AuthenticationError::from(config).into(), req))
}
