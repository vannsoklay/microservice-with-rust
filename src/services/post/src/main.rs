use crate::handlers::{
    create_post, delete_post, get_all_posts, get_all_posts_by_user, get_post_by_user, update_post,
};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use db::DBConfig;
use models::{Post, User};

mod db;
mod handlers;
mod identify;
mod middleware;
mod models;
mod response;
mod utils;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8088")]
    port: String,
}

pub struct AppState {
    pub post_db: mongodb::Collection<Post>,
    pub user_db: mongodb::Collection<User>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args: Cli = Cli::parse();

    let post_db = DBConfig::post_collection().await;
    let user_db = DBConfig::user_collection().await;

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Create AppState
    let app_state = web::Data::new(AppState { post_db, user_db });

    let public_paths = vec!["/api/v1/posts/all".to_string()];

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::AuthMiddleware::new(public_paths.clone()))
            .service(
                web::scope("/api/v1/posts")
                    .route("/all", web::get().to(get_all_posts))
                    .route("", web::get().to(get_all_posts_by_user))
                    .route("", web::post().to(create_post))
                    .route("/{id}", web::get().to(get_post_by_user))
                    .route("/{id}", web::put().to(update_post))
                    .route("/{id}", web::delete().to(delete_post)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
