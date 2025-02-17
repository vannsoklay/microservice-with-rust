use crate::handlers::{
    create_booking, delete_booking, get_all_bookings, get_booking_by_id, update_booking,
};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use db::DBConfig;
use models::Booking;

mod db;
mod handlers;
mod identify;
mod models;
mod response;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8084")]
    port: String,
}

pub struct AppState {
    pub config_db: mongodb::Collection<Booking>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let config_db = DBConfig::booking_collection().await;
    
    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Create AppState
    let app_state = web::Data::new(AppState { config_db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/bookings", web::get().to(get_all_bookings))
            .route("/bookings/{id}", web::get().to(get_booking_by_id))
            .route("/bookings", web::post().to(create_booking))
            .route("/bookings/{id}", web::put().to(update_booking))
            .route("/bookings/{id}", web::delete().to(delete_booking))
    })
    .bind(&bind_address)?
    .run()
    .await
}
