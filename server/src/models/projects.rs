use crate::errors::AppError;
use crate::models::{Result, users::User, repositories::Repository, designs::*};
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Serialize, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Repository, foreign_key = repo_id))]
#[diesel(belongs_to(Design))]
#[diesel(table_name = projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub repo_id: Option<Uuid>,
    pub design_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = projects)]
pub struct NewProject<'a> {
    pub name: &'a str,
    pub repo_id: Option<Uuid>,
    pub design_id: Uuid,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = projects)]
pub struct UpdateProject<'a> {
    pub name: &'a str,
    pub repo_id: Option<Uuid>,
}

pub enum ProjectKey<'a> {
    ID(Uuid),
    Name(&'a str),
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Project))]
#[diesel(table_name = users_projects)]
#[diesel(primary_key(user_id, project_id))]
pub struct UserProject {
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = users_projects)]
pub struct NewUserProject {
    pub user_id: Uuid,
    pub project_id: Uuid,
}

pub fn create_project(conn: &mut PgConnection, project_name: &str, repository_id: Option<Uuid>, user_id: Uuid) -> Result<Project> {
    use crate::schema::projects::dsl::*;

    conn.transaction(|conn| {
        let design = crate::models::designs::create_design(conn)?;
        let insert_project = NewProject{
            name: project_name,
            repo_id: repository_id,
            design_id: design.id,
        };

        let project = diesel::insert_into(projects)
            .values(&insert_project)
            .returning(Project::as_returning())
            .get_result::<Project>(conn)
            .map_err(AppError::from)?;

        register_user_project(conn, NewUserProject { user_id, project_id: project.id })?;

        Ok(project)
    })
}

pub fn find_project<'a>(conn: &mut PgConnection, key: ProjectKey<'a>) -> Result<Project> {
    use crate::schema::projects::dsl::*;

    match key {
        ProjectKey::Name(n) => projects
            .filter(name.eq(n))
            .select(Project::as_select())
            .first(conn)
            .map_err(AppError::from),
        ProjectKey::ID(uuid) => projects
            .find(uuid)
            .select(Project::as_select())
            .first(conn)
            .map_err(AppError::from),
    }
}

pub fn get_user_projects(conn: &mut PgConnection, user_token: &str) -> Result<Vec<Project>> {
    use crate::schema::projects::dsl::*;
    use crate::schema::users::dsl::*;

    conn.transaction(|conn| {
        let user = users
            .filter(last_token.eq(user_token))
            .select(User::as_select())  
            .first::<User>(conn)?;

        UserProject::belonging_to(&user)
            .inner_join(projects)
            .select(Project::as_select())
            .load(conn)
            .map_err(AppError::from)
    })
}

pub fn update_project(conn: &mut PgConnection, project_id: Uuid, new_project: UpdateProject) -> Result<Project> {
    use crate::schema::projects::dsl::*;

    diesel::update(projects)
        .filter(id.eq(project_id))
        .set(&new_project)
        .returning(Project::as_returning())
        .get_result(conn)
        .map_err(AppError::from)
}

fn register_user_project(conn: &mut PgConnection, user_project: NewUserProject) -> Result<usize> {
    use crate::schema::users_projects::dsl::*;

    diesel::insert_into(users_projects)
        .values(&user_project)
        .execute(conn)
        .map_err(AppError::from)
}
