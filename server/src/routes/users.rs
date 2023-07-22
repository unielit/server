use crate::{
    models::{
        users::{self, NewUser},
        Result,
    },
    routes::success,
    DbPool,
};
use actix_web::{web, HttpRequest, Responder};
use utoipa::ToSchema;
use uuid::*;

use super::parse_auth_token;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInput {
    name: String,
    role_id: Uuid,
    email: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(
                web::resource("")
                    .route(web::post().to(create_user))
                    .route(web::patch().to(update_user)),
            )
            .service(web::resource("/find").route(web::get().to(find_user_by_token)))
            .service(web::resource("/find/{name}").route(web::get().to(find_user)))
            .service(web::resource("/roles").route(web::get().to(get_user_roles)))
            .service(web::resource("/{id}").route(web::get().to(get_user)))
            .service(web::resource("/{id}/token").route(web::patch().to(update_user_token))),
    );
}

/// Create a user
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided will be associated with a user account.
#[utoipa::path(
    post,
    context_path = "/users",
    path = "",
    tag = "Users",
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Unique constaint violation."),
        (status = NOT_FOUND, description = "User role record is not found by provided ID."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    request_body(content = UserInput, description = "Input User in JSON format", content_type = "application/json"),
    security(
        ("http" = [])
    )
)]
async fn create_user(
    user: web::Json<UserInput>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token: String = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        users::create_user(
            &mut conn,
            NewUser {
                name: &user.name,
                role_id: user.role_id,
                email: &user.email,
                last_token: Some(&token),
            },
        )
    })
    .await?
    .map(success)
}

/// Find a user by name
///
#[utoipa::path(
    get,
    context_path = "/users",
    path = "/find/{name}",
    tag = "Users",
    params(
        ("name" = String, Path, description = "User's name"),
    ),
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Incorrect name format"),
        (status = NOT_FOUND, description = "User is not found by provided name.")
    )
)]
async fn find_user(name: web::Path<String>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(&mut conn, users::UserKey::Name(&name))
    })
    .await?
    .map(success)
}

/// Find a user by token
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
#[utoipa::path(
    get,
    context_path = "/users",
    path = "/find",
    tag = "Users",
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Incorrect token format"),
        (status = NOT_FOUND, description = "User is not found by provided token."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    security(
        ("http" = [])
    )
)]
async fn find_user_by_token(req: HttpRequest, pool: web::Data<DbPool>) -> Result<impl Responder> {
    let token: String = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(&mut conn, users::UserKey::Token(&token))
    })
    .await?
    .map(success)
}

/// Get a user by id
///
#[utoipa::path(
    get,
    context_path = "/users",
    path = "/{id}",
    tag = "Users",
    params(
        ("id" = Uuid, Path, description = "User's id"),
    ),
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Incorrect id format"),
        (status = NOT_FOUND, description = "User is not found by provided id.")
    )
)]
async fn get_user(id: web::Path<Uuid>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(&mut conn, users::UserKey::ID(id.into_inner()))
    })
    .await?
    .map(success)
}

/// Update a user
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
#[utoipa::path(
    patch,
    context_path = "/users",
    path = "",
    tag = "Users",
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Incorrect input data"),
        (status = NOT_FOUND, description = "User record not found by provided token."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    request_body(content = UserInput, description = "Input User in JSON format", content_type = "application/json"),
    security(
        ("http" = [])
    )
)]
async fn update_user(
    user: web::Json<UserInput>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;
        let user = user.into_inner();

        users::update_user(
            &mut conn,
            &token,
            NewUser {
                name: &user.name,
                role_id: user.role_id,
                email: &user.email,
                last_token: Some(&token),
            },
        )
    })
    .await?
    .map(success)
}

/// Update a user token
///
/// A User Bearer access token should be provided to create a record. 
/// The access token provided must be associated with a user account.
#[utoipa::path(
    patch,
    context_path = "/users",
    path = "/{id}/token",
    tag = "Users",
    responses(
        (status = OK, body = RoledUser),
        (status = BAD_REQUEST, description = "Incorrect input data"),
        (status = NOT_FOUND, description = "User record not found by provided id."),
        (status = UNAUTHORIZED, description = "User is not authorized. Pass user's access token.")
    ),
    params(
        ("id" = Uuid, Path, description = "User's id"),
    ),
    security(
        ("http" = [])
    )
)]
async fn update_user_token(
    id: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        users::update_user_token(&mut conn, id.into_inner(), &token)
    })
    .await?
    .map(success)
}

/// Get all user roles
///
/// Provides all available roles for any user
#[utoipa::path(
    get,
    context_path = "/users",
    path = "/roles",
    tag = "Users",
    responses(
        (status = OK, body = Vec<UserRole>),
    ),
)]
async fn get_user_roles(pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::get_user_roles(&mut conn)
    })
    .await?
    .map(success)
}
