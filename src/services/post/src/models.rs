use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::utils::generate_permlink;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostType {
    SocialMedia,
    Blog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub _id: Option<String>,
    pub title: Option<String>,
    pub permlink: Option<String>,
    pub content: String,
    pub author: Option<String>,
    pub images_url: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub post_type: PostType,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

impl Post {
    pub fn new(
        content: String,
        author: String,
        post_type: PostType,
        title: Option<String>,
        images_url: Vec<String>,
        tags: Option<Vec<String>>,
    ) -> Self {
        let permlink = generate_permlink(title.as_ref());
        Self {
            _id: Some(ObjectId::new().to_hex()),
            title,
            content,
            permlink: Some(permlink),
            author: Some(author),
            images_url,
            tags,
            post_type,
            created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            updated_at: None,
            deleted_at: None
        }
    }

    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.updated_at = Some(DateTime::now().try_to_rfc3339_string().unwrap());
    }
}
