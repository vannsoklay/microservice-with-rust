use crate::handlers::{
    create_accommodation, delete_accommodation, get_accommodation_by_id, get_all_accommodations,
    list_accommodations, update_accommodation,
};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use mongodb::{Client, Database};

mod db;
mod handlers;
mod identify;
mod models;
mod response;

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
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let db = client.database("microservice-db");

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Create AppState
    let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new().app_data(app_state.clone()).service(
            web::scope("/api/v1/accommodations")
                .route("/", web::get().to(get_all_accommodations))
                .route("/list", web::get().to(list_accommodations))
                .route("/{id}", web::get().to(get_accommodation_by_id))
                .route("/", web::post().to(create_accommodation))
                .route("/{id}", web::put().to(update_accommodation))
                .route("/{id}", web::delete().to(delete_accommodation)),
        )
    })
    .bind(&bind_address)?
    .run()
    .await
}
