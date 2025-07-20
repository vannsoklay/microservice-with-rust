mod db;
mod handlers;
mod identify;
mod model;
mod response;
mod storage;
mod stream;
mod utils;

use std::env;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http, web, App, HttpServer};
use clap::Parser;
use db::MongoStorageRepository;
use db::{init_config_db, DBConfig};
use model::FileMetadata;
use storage::LocalStorageService;
use storage::S3StorageService;
use stream::stream_image;

use crate::handlers::{delete_file, upload_file};

pub struct AppState {
    pub db_config: DBConfig<MongoStorageRepository<FileMetadata>>,
    pub local_storage_service: LocalStorageService,
    pub s3_storage_service: S3StorageService,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "9000".into());
    let local_path = env::var("LOCAL_PATH").unwrap_or_else(|_| "./uploads".into());
    let bind_address = format!("127.0.0.1:{}", port);

    println!("Starting server on port {}", port);

    // Initialize DBConfig
    let db_config = init_config_db::<FileMetadata>("file_metadata").await;
    // Initialize AWS S3 client
    let s3_client = db::get_s3_client().await;

    // Initialize StorageService
    let local_storage_service = LocalStorageService::new(local_path); // Directory for uploads
    let s3_storage_service = S3StorageService::new("your-s3-bucket-name".to_string(), s3_client);

    // Create AppState
    let app_state = web::Data::new(AppState {
        db_config,
        local_storage_service,
        s3_storage_service,
    });

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
            .app_data(app_state.clone())
            .route("/storage/local/upload", web::post().to(upload_file))
            .route("/storage/images/{file_name}", web::get().to(stream_image))
            .route("/storage/{id}", web::delete().to(delete_file))
            .wrap(cors)
    })
    .bind(&bind_address)?
    .run()
    .await
}
