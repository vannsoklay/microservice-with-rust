use mongodb::bson::{oid::ObjectId, serde_helpers::serialize_object_id_as_hex_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Follow {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,

    pub follower_id: String,  // the one who follows
    pub following_id: String, // the one being followed

    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
