#[macro_use]
extern crate serde_derive;

use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod apidoc;
mod auth;
mod errors;
mod models;
mod routes;
mod schema;
mod services;

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

        println!("Starting http server: 127.0.0.1:{}", self.port);

        HttpServer::new(move || {
            App::new()
                // .wrap(auth_middleware.clone())
                .app_data(web::Data::new(pool.clone()))
                .configure(routes::users::configure)
                .configure(routes::projects::configure)
                .configure(routes::designs::configure)
                .configure(routes::repositories::configure)
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", openapi.clone()),
                )
        })
        .bind(("127.0.0.1", self.port))?
        .run()
        .await
    }
}
