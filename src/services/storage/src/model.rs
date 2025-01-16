use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub name: String,
    pub url: String,
    pub size: u64,
    pub content_type: String,
    pub location: String,
}