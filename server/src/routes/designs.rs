use crate::{
    errors::AppError,
    models::{designs, projects, Result},
    routes::success,
    DbPool,
};
use actix_web::{web, HttpRequest, Responder};
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
