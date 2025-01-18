mod auth;
mod health;
mod middleware;
mod routing;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpResponse, HttpServer};
use auth::login;
use health::health_check;
use middleware::AuthMiddleware;
use routing::{load_balancer::proxy_request, ServiceState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            .route("/health", web::get().to(health_check)) // Health check endpoint
            .service(login::login)
            .wrap(AuthMiddleware) // Register the authentication middleware
            .route("/{tail:.*}", web::route().to(proxy_request))
            .default_service(web::route().to(|| HttpResponse::NotFound()))
            .wrap(cors)
    })
    .bind("0.0.0.0:8443")?
    .run()
    .await
}
