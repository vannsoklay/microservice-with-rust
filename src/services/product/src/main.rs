mod db;
mod handlers;
mod identify;
mod models;
mod response;
mod utils;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use db::DBConfig;
use handlers::product::{
    create_product, delete_product, get_all_products, get_product_by_id, update_product,
};
use models::product::Product;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8089")]
    port: String,
}

pub struct AppState {
    pub product_config_db: mongodb::Collection<Product>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let product_config_db = DBConfig::post_collection().await;

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Create AppState
    let app_state = web::Data::new(AppState { product_config_db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/products", web::get().to(get_all_products))
            .route("/products/{id}", web::get().to(get_product_by_id))
            .route("/products", web::post().to(create_product))
            .route("/products/{id}", web::put().to(update_product))
            .route("/products/{id}", web::delete().to(delete_product))
    })
    .bind(&bind_address)?
    .run()
    .await
}
