use crate::{
    handlers::{
        follow, follow_count, follow_status, follow_toggle, followers, following, unfollow,
    },
    models::{Follow, User},
};
use actix_web::{App, HttpServer, middleware::Logger, web};
use clap::Parser;
use db::DBConfig;
use mongodb::Collection;

mod db;
mod handlers;
mod middleware;
mod models;
#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "9011")]
    port: String,
}

pub struct AppState {
    pub follow_db: Collection<Follow>,
    pub user_db: Collection<User>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Cli::parse();

    let follow_db = DBConfig::follow_collection().await;
    let user_db = DBConfig::user_collection().await;

    let port = args.port;
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    let app_state = web::Data::new(AppState { follow_db, user_db });

    let public_paths = vec![
        "/api/v1/follow/followers".to_string(),
        "/api/v1/follow/following".to_string(),
        "/api/v1/follow/status".to_string(),
        "/api/v1/follow/counts".to_string(),
    ];

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(middleware::AuthMiddleware::new(public_paths.clone()))
            .service(
                web::scope("/api/v1/follow")
                    .route("/", web::post().to(follow))
                    .route("/unfollow", web::post().to(unfollow))
                    .route("/toggle", web::post().to(follow_toggle))
                    .route("/status", web::get().to(follow_status))
                    .route("/followers/{user_id}", web::get().to(followers))
                    .route("/following/{user_id}", web::get().to(following))
                    .route("/counts/{user_id}", web::get().to(follow_count)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
