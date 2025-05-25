use mongodb::bson::{
    oid::ObjectId,
    serde_helpers::{
        serialize_object_id_as_hex_string,
    },
    DateTime,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    pub permalink: String, // foreign ref to post
    pub author_id: String, // ref to user
    pub vote_type: String, // "up" or "down"
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

impl Vote {
    pub fn insert_body(permalink: String, author_id: String, vote_type: String) -> Vote {
        Vote {
            id: ObjectId::new(),
            permalink,
            author_id,
            vote_type,
            created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            updated_at: None,
            deleted_at: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteReq {
    pub permalink: String,
    pub vote_type: Option<String>,
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
}


#[derive(Debug, Deserialize)]
pub struct VoteQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub sort_by: Option<String>,    // e.g. "created_at"
    pub sort_order: Option<i32>,    // 1 = ascending, -1 = descending
}