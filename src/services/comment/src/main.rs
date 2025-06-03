use crate::handlers::{create_comment, get_comments_by_post};
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use db::DBConfig;
use models::{Comment, Post};
use mongodb::Collection;

mod db;
mod handlers;
mod middleware;
mod models;
#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8099")]
    port: String,
}

pub struct AppState {
    pub comment_db: Collection<Comment>,
    pub post_db: Collection<Post>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Cli::parse();

    let comment_db = DBConfig::comment_collection().await;
    let post_db = DBConfig::post_collection().await;

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    let app_state = web::Data::new(AppState { comment_db, post_db });

    let public_paths = vec!["/api/v1/comments/".to_string()];

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(middleware::AuthMiddleware::new(public_paths.clone()))
            .service(
                web::scope("/api/v1/comments")
                    .route("", web::post().to(create_comment))
                    .route("/{permalink}", web::get().to(get_comments_by_post)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
