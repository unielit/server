use crate::{
    models::{users, users::NewUser, Result},
    routes::success,
    DbPool,
};
use actix_web::{web, HttpRequest, Responder};
use uuid::*;

use super::parse_auth_token;

#[derive(Debug, Serialize, Deserialize)]
struct UserInput {
    name: String,
    role_id: Uuid,
    email: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(web::resource("").route(web::post().to(create_user)))
            .service(web::resource("/find").route(web::get().to(find_user_by_token)))
            .service(web::resource("/find/{name}").route(web::get().to(find_user)))
            .service(web::resource("/{id}").route(web::get().to(get_user)))
            .service(web::resource("/{id}").route(web::patch().to(update_user)))
            .service(web::resource("/{id}/token").route(web::patch().to(update_user_token))),
    );
}

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

async fn find_user(name: web::Path<String>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(&mut conn, users::UserKey::Name(&name))
    })
    .await?
    .map(success)
}

async fn find_user_by_token(req: HttpRequest, pool: web::Data<DbPool>) -> Result<impl Responder> {
    let token: String = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(&mut conn, users::UserKey::Token(&token))
    })
    .await?
    .map(success)
}

async fn get_user(id: web::Path<Uuid>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(&mut conn, users::UserKey::ID(id.into_inner()))
    })
    .await?
    .map(success)
}

async fn update_user(
    id: web::Path<Uuid>,
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
            id.into_inner(),
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