use actix_web::{web, App, HttpServer};
use clap::Parser;
use crate::handlers::{get_all_accommodations, get_accommodation_by_id, create_accommodation, update_accommodation, delete_accommodation};
use mongodb::{Database, Client};

mod models;
mod handlers;
mod response;
mod identify;
mod db;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8081")]
    port: String,
}

pub struct AppState {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    // Initialize MongoDB client
    let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
    let db = client.database("microservice-db");

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Create AppState
    let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/accommodation/all", web::get().to(get_all_accommodations))
            .route("/accommodation/{id}", web::get().to(get_accommodation_by_id))
            .route("/accommodation/create", web::post().to(create_accommodation))
            .route("/accommodation/{id}", web::put().to(update_accommodation))
            .route("/accommodation/{id}", web::delete().to(delete_accommodation))
    })
    .bind(&bind_address)?
    .run()
    .await
    
}
