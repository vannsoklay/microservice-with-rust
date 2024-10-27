use actix_web::{web, App, HttpServer};
use clap::Parser;
use crate::handlers::{get_all_products, get_product_by_id, create_product, update_product, delete_product};

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    HttpServer::new(move || {
        App::new()
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
