use crate::{
    handlers::{create_comment, get_comments_by_post, update_comment, delete_comment},
    models::User,
};
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::env;
use db::DBConfig;
use models::{Comment, Post};
use mongodb::Collection;

mod db;
mod handlers;
mod middleware;
mod models;

pub struct AppState {
    pub comment_db: Collection<Comment>,
    pub post_db: Collection<Post>,
    pub user_db: Collection<User>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8083".into());

    let comment_db = DBConfig::comment_collection().await;
    let post_db = DBConfig::post_collection().await;
    let user_db = DBConfig::user_collection().await;

    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    let app_state = web::Data::new(AppState {
        comment_db,
        post_db,
        user_db,
    });

    let public_paths = vec!["/api/v1/comments/get-post-comments".to_string()];

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(middleware::AuthMiddleware::new(public_paths.clone()))
            .service(
                web::scope("/api/v1/comments")
                    .route("", web::post().to(create_comment))
                    .route(
                        "/get-post-comments/{permalink}",
                        web::get().to(get_comments_by_post),
                    )
                    .route("/{comment_id}", web::put().to(update_comment))
                    .route("/{comment_id}", web::delete().to(delete_comment)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
