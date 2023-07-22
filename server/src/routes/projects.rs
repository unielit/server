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
use utoipa::ToSchema;

use super::parse_auth_token;

#[derive(Deserialize, ToSchema, Debug)]
pub struct InputProject {
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

/// Create a project
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
#[utoipa::path(
    post,
    context_path = "/projects",
    path = "",
    tag = "Projects",
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Unique constaint violation."),
        (status = NOT_FOUND, description = "User is not found by provided token."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    request_body(content = InputProject, description = "Input Project in JSON format", content_type = "application/json"),
    security(
        ("http" = [])
    )
)]
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

/// Find a user by name
///
#[utoipa::path(
    get,
    context_path = "/projects",
    path = "/find/{name}",
    tag = "Projects",
    params(
        ("name" = String, Path, description = "Project name"),
    ),
    responses(
        (status = OK, body = Project),
        (status = BAD_REQUEST, description = "Incorrect name format"),
        (status = NOT_FOUND, description = "Project is not found by provided name.")
    )
)]
async fn find_project(name: web::Path<String>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        projects::find_project(&mut conn, projects::ProjectKey::Name(&name.to_owned()))
    })
    .await?
    .map(success)
}

/// Get a project by id
///
#[utoipa::path(
    get,
    context_path = "/projects",
    path = "/{id}",
    tag = "Projects",
    params(
        ("id" = Uuid, Path, description = "Project record id in database"),
    ),
    responses(
        (status = OK, body = Project),
        (status = BAD_REQUEST, description = "Incorrect data format"),
        (status = NOT_FOUND, description = "Project is not found by provided id"),
    )
)]
async fn get_project(id: web::Path<Uuid>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        projects::find_project(&mut conn, projects::ProjectKey::ID(id.to_owned()))
    })
    .await?
    .map(success)
}

/// Get the projects by user token
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
#[utoipa::path(
    get,
    context_path = "/projects",
    path = "",
    tag = "Projects",
    responses(
        (status = OK, body = Vec<Project>),
        (status = BAD_REQUEST, description = "Incorrect data format"),
        (status = NOT_FOUND, description = "User is not found by provided token."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    security(
        ("http" = [])
    )
)]
async fn get_user_projects(req: HttpRequest, pool: web::Data<DbPool>) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        projects::get_user_projects(&mut conn, &token)
    })
    .await?
    .map(success)
}

/// Update a project
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
#[utoipa::path(
    patch,
    context_path = "/projects",
    path = "/{id}",
    tag = "Projects",
    params(
        ("id" = Uuid, Path, description = "Project record id in database"),
    ),
    request_body(content = InputProject, description = "Input Project in JSON format", content_type = "application/json"),
    responses(
        (status = OK, body = Project),
        (status = BAD_REQUEST, description = "Incorrect data format"),
        (status = NOT_FOUND, description = "Either User is not found by provided token or project record to update."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    security(
        ("http" = [])
    )
)]
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
