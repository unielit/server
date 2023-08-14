use crate::{models, routes};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::designs::get_design,
        routes::designs::update_design,
        routes::users::create_user,
        routes::users::find_user,
        routes::users::find_user_by_token,
        routes::users::get_user,
        routes::users::update_user,
        routes::users::update_user_token,
        // routes::users::get_user_roles,
        routes::repositories::create_repo,
        routes::repositories::save_repo_design,
        routes::projects::create_project,
        routes::projects::find_project,
        routes::projects::get_project,
        routes::projects::get_user_projects,
        routes::projects::update_project,
        routes::auth::github::generate_access_token,
    ),
    components(
        schemas(
            models::designs::Design, 
            models::users::RoledUser, 
            // models::users::UserRole,
            models::users::User,
            routes::users::UserInput,
            models::repositories::Repository,
            routes::repositories::InputRepository,
            routes::repositories::SaveRepoDesign,
            models::projects::Project,
            routes::projects::InputProject,
        )
    ),
    tags(
        (name = "Designs", description = "Design management endpoints."),
        (name = "Users", description = "Users management endpoints."),
        (name = "Repositories", description = "Repositories management endpoints."),
        (name = "Projects", description = "Projects management endpoints."),
        (name = "Auth Github", description = "Github Auth management endpoints."),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "http",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}
