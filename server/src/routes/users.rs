use crate::{
    models::{users, users::NewUser, Result},
    routes::success,
    DbPool,
};
use actix_web::{web, Responder, HttpRequest};
use uuid::*;

use super::parse_auth_token;

#[derive(Debug, Serialize, Deserialize)]
struct UserInput {
    name: String,
    role_id: String,
    email: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(web::resource("").route(web::post().to(create_user)))
            .service(web::resource("/find/{email}").route(web::get().to(find_user)))
            .service(web::resource("/{id}").route(web::get().to(get_user)))

    );
}

async fn create_user(
    user: web::Json<UserInput>,
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let token = parse_auth_token(req)?;

    web::block(move || {
        let mut conn = pool.get()?;
        let role_id = Uuid::parse_str(&user.role_id)?;

        users::create_user(
            &mut conn,
            NewUser {
                name: &user.name,
                role_id,
                email: &user.email,
                last_token: Some(&token),
            },
        )
    })
    .await?
    .map(success)
}

async fn find_user(email: web::Path<String>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(
            &mut conn,
            users::UserKey::Email(&email),
        )
    })
    .await?
    .map(success)
}

async fn get_user(id: web::Path<Uuid>, pool: web::Data<DbPool>) -> Result<impl Responder> {
    web::block(move || {
        let mut conn = pool.get()?;

        users::find_user(
            &mut conn,
            users::UserKey::ID(id.into_inner()),
        )
    })
    .await?
    .map(success)
}