use crate::errors::AppError;
use crate::models::Result;
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(UserRole, foreign_key = role_id))]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub role_id: Uuid,
    pub email: String,
    pub last_token: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub role_id: Uuid,
    pub email: &'a str,
}

pub enum UserKey<'a> {
    ID(Uuid),
    Email(&'a str),
}

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = user_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRole {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct RoledUser {
    pub user: User,
    pub role: UserRole,
}

pub fn create_user(conn: &mut PgConnection, new_user: NewUser) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user = diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(conn)
            .map_err(AppError::from)?;

        let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user, role })
    })
}

pub fn find_user<'a>(conn: &mut PgConnection, key: UserKey<'a>) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user: User = match key {
            UserKey::Email(em) => users
                .filter(email.eq(em))
                .select(User::as_select())
                .first(conn)
                .map_err(AppError::from),
            UserKey::ID(uuid) => users
                .find(uuid)
                .select(User::as_select())
                .first(conn)
                .map_err(AppError::from),
        }?;

        let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user, role })
    })
}

pub fn update_user_token(
    conn: &mut PgConnection,
    user_email: &str,
    token: &str,
) -> Result<RoledUser> {
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user = diesel::update(users)
            .filter(email.eq(user_email))
            .set(last_token.eq(token))
            .returning(User::as_returning())
            .get_result(conn)
            .map_err(AppError::from)?;

        let role = find_role(conn, user.role_id)?;
        Ok(RoledUser { user, role })
    })
}

fn find_role(conn: &mut PgConnection, role_id: Uuid) -> Result<UserRole> {
    use crate::schema::user_roles::dsl::*;

    user_roles
        .find(role_id)
        .select(UserRole::as_select())
        .first(conn)
        .map_err(AppError::from)
}
