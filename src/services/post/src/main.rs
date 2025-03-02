use crate::handlers::{create_post, delete_post, get_all_posts, get_post_by_id, update_post};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use db::DBConfig;
use models::Post;

mod db;
mod handlers;
mod identify;
mod models;
mod response;
mod utils;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8088")]
    port: String,
}

pub struct AppState {
    pub config_db: mongodb::Collection<Post>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let config_db = DBConfig::post_collection().await;

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Create AppState
    let app_state = web::Data::new(AppState { config_db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/posts", web::get().to(get_all_posts))
            .route("/posts/{id}", web::get().to(get_post_by_id))
            .route("/posts", web::post().to(create_post))
            .route("/posts/{id}", web::put().to(update_post))
            .route("/posts/{id}", web::delete().to(delete_post))
    })
    .bind(&bind_address)?
    .run()
    .await
}
