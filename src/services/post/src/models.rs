use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::utils::generate_permalink;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostType {
    SocialMedia,
    Blog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub title: Option<String>,
    pub permalink: Option<String>,
    pub content: String,
    pub author_id: Option<String>,
    pub media_urls: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub post_type: PostType,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
}

impl Post {
    pub fn new(
        content: String,
        author_id: Option<String>,
        post_type: PostType,
        title: Option<String>,
        media_urls: Vec<String>,
        tags: Option<Vec<String>>,
    ) -> Self {
        let permalink = generate_permalink(title.as_ref());
        Self {
            id: Some(ObjectId::new().to_hex()),
            title,
            content,
            permalink: Some(permalink),
            author_id,
            media_urls,
            tags,
            post_type,
            created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            updated_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            deleted_at: None,
        }
    }
}
