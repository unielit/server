use crate::errors::AppError;
use crate::models::Result;
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::dsl::exists;
use diesel::prelude::*;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Serialize, ToSchema, Debug, PartialEq)]
#[diesel(table_name = repositories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
    pub html_url: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct RepositoryOwner {
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
}

#[derive(Insertable, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = repositories)]
pub struct NewRepository {
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
    pub html_url: String,
}

pub enum RepositoryKey {
    ID(Uuid),
    Owner(RepositoryOwner),
}

pub fn create_repo(conn: &mut PgConnection, new_repo: NewRepository) -> Result<Repository> {
    use crate::schema::repositories::dsl::*;

    diesel::insert_into(repositories)
        .values(&new_repo)
        .returning(Repository::as_returning())
        .get_result::<Repository>(conn)
        .map_err(AppError::from)
}

pub fn is_repo_exist(conn: &mut PgConnection, repo_owner: RepositoryOwner) -> Result<bool> {
    use crate::schema::repositories::dsl::*;

    diesel::select(exists(
        repositories.filter(
            name.eq(repo_owner.name)
                .and(owner.eq(repo_owner.owner))
                .and(is_organization.eq(repo_owner.is_organization)),
        ),
    ))
    .get_result::<bool>(conn)
    .map_err(AppError::from)
}
