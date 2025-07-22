use mongodb::bson::{oid::ObjectId, serde_helpers::serialize_object_id_as_hex_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Follow {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,

    pub follower_id: String,  // the one who follows
    pub following_id: String, // the one being followed

    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FollowRequest {
    pub following_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Query {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub sort_by: Option<String>, // e.g. "created_at"
    pub sort_order: Option<i32>, // 1 = ascending, -1 = descending
}

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub follower_id: String,
    pub following_id: String,
}
