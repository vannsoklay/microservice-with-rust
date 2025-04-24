use crate::handlers::{change_password, get_user};
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use mongodb::{Client, Database};

mod db;
mod handlers;
mod middleware;
mod models;
mod response;
#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "8083")]
    port: String,
}

pub struct AppState {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Cli::parse();

    // Initialize MongoDB client
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let db = client.database("microservice-db");

    let port = args.port;
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
