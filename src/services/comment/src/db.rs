use mongodb::{options::ClientOptions, Client, Collection, Database};

pub struct DBConfig {}

use crate::models::{Post, Comment};

async fn db() -> Database {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();

    let client = Client::with_options(client_options).unwrap();
    let database = client.database("microservice-db");
    database
}

impl DBConfig {
    pub async fn comment_collection() -> Collection<Comment> {
        db().await.collection::<Comment>("comments")
    }

    pub async fn post_collection() -> Collection<Post> {
        db().await.collection::<Post>("posts")
    }
}
