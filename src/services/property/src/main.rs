use crate::handlers::{
    create_property, delete_property, get_all_properties, get_property_by_id, update_property,
};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use db::DBConfig;
use models::Property;

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
    pub config_db: mongodb::Collection<Property>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let config_db = DBConfig::property_collection().await;

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    let app_state = web::Data::new(AppState { config_db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/properties", web::get().to(get_all_properties))
            .route("/properties/{id}", web::get().to(get_property_by_id))
            .route("/properties", web::post().to(create_property))
            .route("/properties/{id}", web::put().to(update_property))
            .route("/properties/{id}", web::delete().to(delete_property))
    })
    .bind(&bind_address)?
    .run()
    .await
}
