use mongodb::{options::ClientOptions, Client, Collection, Database};

pub struct DBConfig {}

use crate::models::Booking;

async fn db() -> Database {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();

    let client = Client::with_options(client_options).unwrap();

    // Ping the database to confirm connection
    let database = client.database("admin");
    database
        .run_command(mongodb::bson::doc! { "ping": 1 })
        .await
        .unwrap();

    let database = client.database("microservice-db");
    database
}

impl DBConfig {
    pub async fn booking_collection() -> Collection<Booking> {
        db().await.collection::<Booking>("bookings")
    }
}
