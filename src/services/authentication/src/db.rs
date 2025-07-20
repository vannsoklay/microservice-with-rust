use mongodb::{options::ClientOptions, Client, Collection, Database};
use std::env;

pub struct DBConfig {}

use crate::models::User;

pub async fn db() -> Database {
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

    let client = Client::with_options(client_options).unwrap();
    let database = client.database(&db_name);
    database
}

impl DBConfig {
    pub async fn user_collection() -> Collection<User> {
        db().await.collection::<User>("users")
    }
}
