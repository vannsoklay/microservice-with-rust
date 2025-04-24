mod auth;
mod handler;
mod health;
mod middleware;
mod routing;
mod utils;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use handler::forward_request;
use health::health_check;
use middleware::jwt::JwtMiddleware;
use routing::ServiceState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let state = Arc::new(ServiceState::new());

    HttpServer::new(move || {
        let cors = Cors::permissive()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health_check))
            .wrap(JwtMiddleware {
                secret: std::env::var("JWT_SECRET").expect("JWT_SECRET missing"),
            })
            .route("/api/v1/{tail:.*}", web::route().to(forward_request))
            .default_service(web::route().to(|| HttpResponse::NotFound()))
            .wrap(cors)
    })
    .bind("0.0.0.0:8443")?
    .run()
    .await
}
