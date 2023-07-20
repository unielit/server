use crate::{
    models::{
        projects::{self, UpdateProject},
        users, Result,
    },
    routes::success,
    DbPool,
};
use actix_web::{web, HttpRequest, Responder};
use uuid::*;

use super::parse_auth_token;

#[derive(Deserialize, Debug)]
struct InputProject {
    name: String,
    repo_id: Option<Uuid>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/projects")
            .service(
                web::resource("")
                    .route(web::get().to(get_user_projects))
                    .route(web::post().to(create_project)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_project))
                    .route(web::patch().to(update_project)),
            )
            .service(web::resource("/find/{name}").route(web::get().to(find_project))),
    );
}

async fn create_project(
    input: web::Json<InputProject>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;
        let roled_user = users::find_user(&mut conn, users::UserKey::Token(&token))?;

        projects::create_project(&mut conn, &input.name, input.repo_id, roled_user.user.id)
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

async fn update_project(
    id: web::Path<Uuid>,
    project: web::Json<InputProject>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        projects::update_project(
            &mut conn,
            id.into_inner(),
            UpdateProject {
                name: &project.name,
                repo_id: project.repo_id,
            },
        )
    })
    .await?
    .map(success)
}
