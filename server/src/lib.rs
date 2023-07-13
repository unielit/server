#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix_web::{web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server { port }
    }

    pub async fn run(&self, database_url: String) -> std::io::Result<()> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create PostgreSQL connection pool");
        
        println!("Starting http server: 127.0.0.1:{}", self.port);

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(pool.clone()))
        })
        .bind(("127.0.0.1", self.port))?
        .run()
        .await
    }
}
