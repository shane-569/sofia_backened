mod config;
mod db;
mod models;
mod auth;
mod routes;

use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = db::init().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(routes::auth::init)
            .configure(routes::profile::init) // Now accessible at /me directly
            .wrap(Logger::default())
    })
        .bind("127.0.0.1:8081")?
        .run()
        .await
}