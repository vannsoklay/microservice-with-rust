use actix_web::{web, App, HttpServer};
use clap::Parser;
use crate::handlers::{register, login};
use mongodb::{Database, Client};
   
mod models;
mod handlers;
mod response;
mod identify;
mod db;
mod jwt;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8011")]
    port: String,
}

pub struct AppState {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();


    let args = Cli::parse();

    // Initialize MongoDB client
    // let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
    // let db = client.database("microservice-db");

    let db = db::db().await;

    let port = args.port;
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
