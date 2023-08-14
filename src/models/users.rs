use crate::errors::AppError;
use crate::models::Result;
use crate::schema::*;
use crate::services::encrypt::{EncryptResponse, Aes256Gcm};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use ring::aead::NONCE_LEN;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Queryable, Selectable, Identifiable,
    // Associations, 
    Serialize, ToSchema, Debug, PartialEq,
)]
// #[diesel(belongs_to(UserRole, foreign_key = role_id))]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    // pub role_id: Uuid,
    pub email: String,
    pub access_token: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    // pub role_id: Uuid,
    pub email: String,
    pub access_token: Option<String>,
}

pub enum UserKey<'a> {
    ID(Uuid),
    Name(&'a str),
    Token(&'a str),
}

// #[derive(Queryable, Selectable, Identifiable, Serialize, ToSchema, Debug, PartialEq)]
// #[diesel(table_name = user_roles)]
// #[serde(rename_all = "camelCase")]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct UserRole {
//     pub id: Uuid,
//     pub name: String,
//     pub created_at: NaiveDateTime,
//     pub updated_at: NaiveDateTime,
// }

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoledUser {
    pub user: User,
//     pub role: UserRole,
}

#[derive(
    Queryable, Selectable, Identifiable, Associations, Serialize, ToSchema, Debug, PartialEq,
)]
#[diesel(belongs_to(User, foreign_key = user_id), primary_key(user_id))]
#[diesel(table_name = user_refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRefreshToken {
    pub user_id: Uuid,
    pub refresh_token_cypher: Vec<u8>,
    pub cypher_nonce: Vec<u8>,
    pub refresh_token_expires_in: i32,
    pub scope: String,
    pub token_type: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = user_refresh_tokens)]
struct NewUserRefreshToken {
    pub user_id: Uuid,
    pub refresh_token_cypher: Vec<u8>,
    pub cypher_nonce: Vec<u8>,
    pub refresh_token_expires_in: i32,
    pub scope: String,
    pub token_type: String,
}

pub struct TokenData {
    pub access_token: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub refresh_token_expires_in: i32,
    pub scope: String,
    pub token_type: String,
}

pub fn create_user(conn: &mut PgConnection, new_user: NewUser) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user = diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(conn)
            .map_err(AppError::from)?;

        // let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user/*, role*/ })
    })
}

pub fn find_user(conn: &mut PgConnection, key: UserKey) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user: User = match key {
            UserKey::Token(token) => users
                .filter(access_token.eq(token))
                .select(User::as_select())
                .first(conn)
                .map_err(AppError::from),
            UserKey::Name(n) => users
                .filter(name.eq(n))
                .select(User::as_select())
                .first(conn)
                .map_err(AppError::from),
            UserKey::ID(uuid) => users
                .find(uuid)
                .select(User::as_select())
                .first(conn)
                .map_err(AppError::from),
        }?;

        // let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user/*, role*/ })
    })
}

pub fn update_user(conn: &mut PgConnection, token: &str, new_user: NewUser) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user = diesel::update(users)
            .filter(access_token.eq(token))
            .set(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
            .map_err(AppError::from)?;

        // let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user/*, role*/ })
    })
}

pub fn update_user_token(conn: &mut PgConnection, user_id: Uuid, token: &str) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user = diesel::update(users)
            .filter(id.eq(user_id))
            .set(access_token.eq(token))
            .returning(User::as_returning())
            .get_result(conn)
            .map_err(AppError::from)?;

        // let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user/*, role*/ })
    })
}

// fn find_role(conn: &mut PgConnection, role_id: Uuid) -> Result<UserRole> {
//     use crate::schema::user_roles::dsl::*;

//     user_roles
//         .find(role_id)
//         .select(UserRole::as_select())
//         .first(conn)
//         .map_err(AppError::from)
// }

// pub fn get_user_roles(conn: &mut PgConnection) -> Result<Vec<UserRole>> {
//     use crate::schema::user_roles::dsl::*;

//     user_roles
//         .select(UserRole::as_select())
//         .load(conn)
//         .map_err(AppError::from)
// }

pub fn save_user_token_data(
    conn: &mut PgConnection,
    mut user: NewUser,
    token_data: TokenData,
) -> Result<()> {
    use crate::schema::user_refresh_tokens::dsl::*;
    use crate::schema::users::dsl::*;

    let aes_256_gcm = Aes256Gcm::new();
    let data = token_data.refresh_token.as_bytes().to_vec();
    let encrypt_response: EncryptResponse = aes_256_gcm.encrypt(data, [0u8, 0].to_vec())?;

    user.access_token = Some(token_data.access_token.clone());

    conn.transaction(|conn| {         
        let user = diesel::insert_into(users)
            .values(&user)
            .on_conflict(email)
            .do_update()
            .set(access_token.eq(token_data.access_token))
            .returning(User::as_returning())
            .get_result::<User>(conn)
            .map_err(AppError::from)?;

        let new_token_data = NewUserRefreshToken {
            user_id: user.id,
            refresh_token_cypher: encrypt_response.cypher,
            cypher_nonce: encrypt_response.nonce.to_vec(),
            refresh_token_expires_in: token_data.refresh_token_expires_in,
            scope: token_data.scope,
            token_type: token_data.token_type,
        };

        diesel::insert_into(user_refresh_tokens)
            .values(&new_token_data)
            .on_conflict(user_id)
            .do_update()
            .set(&new_token_data)
            .returning(UserRefreshToken::as_returning())
            .get_result::<UserRefreshToken>(conn)
            .map_err(AppError::from)?;            

        Ok(())
    })
}

pub fn get_user_refresh_token(conn: &mut PgConnection, id: Uuid) -> Result<String> {
    use crate::schema::user_refresh_tokens::dsl::*;

    let data = user_refresh_tokens
        .find(id)
        .select(UserRefreshToken::as_select())
        .first(conn)
        .map_err(AppError::from)?;

    if data.cypher_nonce.len() != NONCE_LEN {
        return Err(AppError::CryptoError("Wrong nonce length during decryption process.".to_string()));
    }
    
    let aes_256_gcm = Aes256Gcm::new();
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&data.cypher_nonce);

    let decrypted_token_data = aes_256_gcm.decrypt(data.refresh_token_cypher, [0u8, 0].to_vec(), nonce)?;

    String::from_utf8(decrypted_token_data)
        .map_err(|e| AppError::CryptoError(format!("Failed to decode token binary data to utf8 string. Error: {}", e)))
}