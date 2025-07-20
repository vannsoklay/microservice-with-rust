use std::env;
use mongodb::{options::ClientOptions, Client, Collection};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::Client as S3Client;
use serde::{Deserialize, Serialize};

pub struct DBConfig<S> {
    pub storage_repo: S,
}

impl<S> DBConfig<S> {
    pub async fn init(storage_repo: S) -> Self {
        Self { storage_repo }
    }
}

pub struct MongoStorageRepository<T>
where
    T: Send + Sync + Unpin + Serialize + for<'de> Deserialize<'de>,
{
    collection: Collection<T>,
}

impl<T> MongoStorageRepository<T>
where
    T: Send + Sync + Unpin + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(collection: Collection<T>) -> Self {
        Self { collection }
    }

    pub fn get_collection(&self) -> &Collection<T> {
        &self.collection
    }
}

pub async fn init_config_db<T>(collection_name: &str) -> DBConfig<MongoStorageRepository<T>>
where
    T: Send + Sync + Unpin + Serialize + for<'de> Deserialize<'de>,
{
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
    //     .expect("Failed to parse MongoDB client options");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let database = client.database(&db_name);
    let collection = database.collection::<T>(collection_name);

    let storage_repo = MongoStorageRepository::new(collection);
    DBConfig::init(storage_repo).await
}

pub async fn get_s3_client() -> S3Client {
    // Automatically load AWS credentials and region from the environment
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28()).region(region_provider).load().await;

    // Create an S3 client
    S3Client::new(&config)
}
