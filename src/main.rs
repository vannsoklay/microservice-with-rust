mod auth;
mod health;
mod middleware;
mod routing;

use std::sync::Arc;

use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use auth::{jwt::{create_token, validate_token}, login};
use health::health_check;
use middleware::AuthMiddleware;
use routing::{load_balancer::proxy_request, ServiceState};
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

// pub async fn login(user_id: web::Json<String>, secret: &str) -> impl Responder {
//     let token = create_token(&user_id, secret);
//     HttpResponse::Ok().body(token)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Arc::new(ServiceState::new());
    // // Set up SSL/TLS
    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder.set_private_key_file("certs/key.pem", SslFiletype::PEM).unwrap();
    // builder.set_certificate_chain_file("certs/cert.pem").unwrap();

    // // Start the HTTP server with SSL
    // HttpServer::new(move || {
    //     App::new()
    //         .wrap(rate_limit::RateLimit::new(100, std::time::Duration::from_secs(60))) // Rate limiting
    //         .route("/health", web::get().to(health_check)) // Health check endpoint
    //         .service(web::resource("/api").route(web::get().to(validate_token))) // Protected API route
    // })
    // .bind_ssl("0.0.0.0:8443", builder)? // Use the configured SSL
    // .run()
    // .await

    // let secret = "your_secret_key"; // Store this in an environment variable or configuration

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default()) // Optional: Log requests
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health_check)) // Health check endpoint
            .service(login::login)
            .wrap(AuthMiddleware) // Register the authentication middleware
            .route("/api", web::route().to(proxy_request))
            .default_service(web::route().to(|| HttpResponse::NotFound()))
    })
    .bind("0.0.0.0:8443")?
    .run()
    .await
}
