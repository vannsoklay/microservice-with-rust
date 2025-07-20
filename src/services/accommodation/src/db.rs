use mongodb::{options::ClientOptions, Client, Collection, Database};
use std::env;

pub struct DBConfig {}

use crate::models::Accommodation;

async fn db() -> Database {
    dotenv::dotenv().ok();

    let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("DB_PORT").unwrap_or_else(|_| "27017".into());
    let username = env::var("DB_USERNAME").unwrap();
    let password = env::var("DB_PASSWORD").unwrap();
    let db_name = env::var("DB_NAME").unwrap_or_else(|_| "admin".into());

    let mongo_uri = format!(
        "mongodb://{}:{}@{}:{}",
        username, password, host, port
    );

    println!("url {:?}", mongo_uri);
    let client_options = ClientOptions::parse(&mongo_uri)
        .await
        .unwrap();

    // let client_options = ClientOptions::parse("mongodb://localhost:27017")
    //     .await
    //     .unwrap();
    
    let client = Client::with_options(client_options).unwrap();

    // Ping the database to confirm connection
    let database = client.database("admin");
    database.run_command(mongodb::bson::doc! { "ping": 1 }).await.unwrap();

    let database = client.database("microservice-db");
    database
}

impl DBConfig {
    pub async fn accommodation_collection() -> Collection<Accommodation> {
        db().await.collection::<Accommodation>("accommodations")
    }
}
