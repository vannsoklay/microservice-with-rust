use actix_web::{web, App, HttpServer};
use std::env;
use crate::handlers::{register, login};
use mongodb::{Database, Client};

mod models;
mod handlers;
mod response;
mod identify;
mod db;
mod jwt;

pub struct AppState {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8081".into());
    let db = db::db().await;

    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

     // Create AppState
     let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/v1/auth/login", web::post().to(login))
            .route("/api/v1/auth/register", web::post().to(register))
    })
    .bind(&bind_address)?
    .run()
    .await
    
}
