use mongodb::bson::{oid::ObjectId, serde_helpers::serialize_object_id_as_hex_string, DateTime};
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    pub permalink: String,
    pub author_id: String,
    pub content: String,
    pub parent_comment_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

impl Comment {
    pub fn insert_body(
        permalink: String,
        author_id: String,
        content: String,
        parent_comment_id: Option<String>,
    ) -> Comment {
        Comment {
            id: ObjectId::new(),
            permalink,
            author_id,
            content,
            parent_comment_id,
            created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            updated_at: None,
            deleted_at: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename(serialize = "id"))]
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub is_verified: bool,
    pub follower_count: i32,
    pub following_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentReq {
    pub permalink: String,
    pub content: String,
    pub parent_comment_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub sort_by: Option<String>, // e.g. "created_at"
    pub sort_order: Option<i32>, // 1 = ascending, -1 = descending
}
