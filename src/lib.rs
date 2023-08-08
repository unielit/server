#[macro_use]
extern crate serde_derive;

use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod apidoc;
mod errors;
mod models;
mod routes;
mod schema;
mod services;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server { port }
    }

    pub async fn run(&self, database_url: String) -> std::io::Result<()> {
        // Middleware for checking our generated tokens from OAuth 2.0.
        // Could be used in future
        let _auth_middleware = HttpAuthentication::bearer(routes::validator);
        let openapi = apidoc::ApiDoc::openapi();
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create PostgreSQL connection pool");

        run_migrations(&pool);
        
        println!("Starting http server: localhost:{}", self.port);

        HttpServer::new(move || {
            App::new()
                // .wrap(auth_middleware.clone())
                .app_data(web::Data::new(pool.clone()))
                .configure(routes::users::configure)
                .configure(routes::projects::configure)
                .configure(routes::designs::configure)
                .configure(routes::repositories::configure)
                .configure(routes::auth::github::configure)
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", openapi.clone()),
                )
        })
        .bind(("0.0.0.0", self.port))?
        .run()
        .await
    }
}

fn run_migrations(pool: &DbPool) {
    let mut conn = pool
        .get()
        .expect("Failed to get connection to PostgreSQL Db pool during migrations");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run diesel PostgreSQL migrations");
}
