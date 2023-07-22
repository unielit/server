use crate::{
    errors::AppError,
    models::{designs, projects, Result},
    routes::success,
    DbPool,
};
use actix_web::{ web, HttpRequest, Responder};
use utoipa::{self};
use uuid::*;

use super::parse_auth_token;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/designs").service(
            web::resource("/{id}")
                .route(web::get().to(get_design))
                .route(web::patch().to(update_design)),
        ),
    );
}

/// Get a design 
/// 
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
/// 
/// The authenticated user must have access to design's project.
#[utoipa::path(
    get,
    context_path = "/designs",
    path = "/{id}",
    tag = "Designs",
    responses(
        (status = OK, body = Design),
        (status = NOT_FOUND),
        (status = FORBIDDEN, description = "Authorized user doesn't have required permission."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    params(
        ("id" = Uuid, Path, description = "Design record id in the database"),
    ),
    security(
        ("http" = [])
    )
)]
async fn get_design(
    id: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token: String = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;
        let projects: Vec<projects::Project> = projects::get_user_projects(&mut conn, &token)?;

        if !projects.iter().any(|p| p.design_id.eq(&id)) {
            return Err(AppError::PermissionError);
        }

        designs::get_design(&mut conn, id.into_inner())
    })
    .await?
    .map(success)
}

/// Update a design 
/// 
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
/// 
/// The authenticated user must have access to design's project.
#[utoipa::path(
    patch,
    context_path = "/designs",
    path = "/{id}",
    tag = "Designs",
    responses(
        (status = OK, body = Design),
        (status = NOT_FOUND),
        (status = FORBIDDEN, description = "Authorized user doesn't have required permission."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    request_body(content = serde_json::Value, description = "Design structure in JSON format", content_type = "application/json"),
    params(
        ("id" = Uuid, Path, description = "Design record id in the database"),
    ),
    security(
        ("http" = [])
    )
)]
async fn update_design(
    id: web::Path<Uuid>,
    data: web::Json<serde_json::Value>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token: String = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;
        let projects: Vec<projects::Project> = projects::get_user_projects(&mut conn, &token)?;

        if !projects.iter().any(|p| p.design_id.eq(&id)) {
            return Err(AppError::PermissionError);
        }

        designs::update_design(&mut conn, id.into_inner(), data.into_inner())
    })
    .await?
    .map(success)
}
