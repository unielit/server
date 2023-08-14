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
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub is_organization: bool,
    pub design_file_sha: Option<String>,
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

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = repositories)]
pub struct NewRepository<'a> {
    pub name: &'a str,
    pub owner: &'a str,
    pub is_organization: bool,
    pub design_file_sha: Option<&'a str>,
    pub html_url: &'a str,
}


#[derive(Insertable, Serialize, Deserialize, AsChangeset, Default, Debug)]
#[diesel(table_name = repositories)]
pub struct UpdateRepository<'a> {
    pub name: Option<&'a str>,
    pub owner: Option<&'a str>,
    pub is_organization: Option<bool>,
    pub design_file_sha: Option<&'a str>,
    pub html_url: Option<&'a str>,
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

pub fn find_repo(conn: &mut PgConnection, key: RepositoryKey) -> Result<Repository> {
    use crate::schema::repositories::dsl::*;

    conn.transaction(|conn| match key {
        RepositoryKey::Owner(repo_owner) => repositories
            .filter(
                owner
                    .eq(repo_owner.owner)
                    .and(name.eq(repo_owner.name))
                    .and(is_organization.eq(repo_owner.is_organization)),
            )
            .select(Repository::as_select())
            .first(conn)
            .map_err(AppError::from),
        RepositoryKey::ID(uuid) => repositories
            .find(uuid)
            .select(Repository::as_select())
            .first(conn)
            .map_err(AppError::from),
    })
}

pub fn update_repo(
    conn: &mut PgConnection,
    repo_id: Uuid,
    upd_repo: UpdateRepository,
) -> Result<Repository> {
    use crate::schema::repositories::dsl::*;

    diesel::update(repositories)
        .filter(id.eq(repo_id))
        .set(&upd_repo)
        .returning(Repository::as_returning())
        .get_result(conn)
        .map_err(AppError::from)
}
