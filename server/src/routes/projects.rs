use crate::{
    models::{projects, projects::NewProject, Result},
    routes::success,
    DbPool,
};
use actix_web::{web, HttpRequest, Responder};
use uuid::*;

use super::parse_auth_token;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/projects")
            .service(web::resource("").route(web::post().to(create_project)))
            .service(web::resource("/{id}").route(web::get().to(get_project)))
            .service(web::resource("/{id}").route(web::patch().to(update_project)))
            .service(web::resource("/find/{name}").route(web::get().to(find_project)))
            .service(web::resource("/find/all").route(web::get().to(get_user_projects)))
    );
}

async fn create_project(
    project: web::Json<NewProject>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        projects::create_project(
            &mut conn,
            project.into_inner(),
            &token,
        )
    })
    .await?
    .map(success)
}

async fn find_project(name: web::Path<String>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        projects::find_project(&mut conn, projects::ProjectKey::Name(&name.to_owned()))
    })
    .await?
    .map(success)
}

async fn get_project(id: web::Path<Uuid>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        projects::find_project(&mut conn, projects::ProjectKey::ID(id.to_owned()))
    })
    .await?
    .map(success)
}

async fn get_user_projects(req: HttpRequest, pool: web::Data<DbPool>) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        projects::get_user_projects(&mut conn, &token)
    })
    .await?
    .map(success)
}

async fn update_project(id: web::Path<Uuid>, project: web::Json<NewProject>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        projects::update_project(
            &mut conn,
            id.into_inner(),
            project.into_inner(),
        )
    })
    .await?
    .map(success)
}
