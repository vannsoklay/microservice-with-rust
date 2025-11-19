use crate::handlers::{change_password, get_user};
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::env;
use mongodb::Database;

mod db;
mod handlers;
mod middleware;
mod models;
mod response;

pub struct AppState {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());

    let db = db::db().await;

    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(middleware::AuthMiddleware::new(vec![]))
            .service(
                web::scope("/api/v1/user")
                    .route("", web::get().to(get_user))
                    .route("/password", web::post().to(change_password)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
