use mongodb::{options::ClientOptions, Client, Collection, Database};

pub struct DBConfig {}

use crate::models::User;

async fn db() -> Database {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();

    let client = Client::with_options(client_options).unwrap();
    let database = client.database("microservice-db");
    database
}

impl DBConfig {
    pub async fn user_collection() -> Collection<User> {
        db().await.collection::<User>("users")
    }
}
