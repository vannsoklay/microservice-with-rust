use actix_web::Result;
use mongodb::bson::{oid::ObjectId, serde_helpers::serialize_object_id_as_hex_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    #[serde(rename(serialize = "id"))]
    #[serde(rename(deserialize = "_id"))]
    pub id: ObjectId,

    username: Option<String>,
    email: Option<String>,
    avatar: Option<String>,
    bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Follow {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,

    pub follower_id: String,  // the one who follows
    pub following_id: String, // the one being followed

    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapFollow {
    pub follower: User,
    pub following: User,
}

impl Follow {
    pub fn get_follower(
        id: ObjectId,
        username: Option<String>,
        email: Option<String>,
        avatar: Option<String>,
        bio: Option<String>,
    ) -> User {
        User {
            id,
            username,
            email,
            avatar,
            bio,
        }
    }
    pub fn get_following(
        id: ObjectId,
        username: Option<String>,
        email: Option<String>,
        avatar: Option<String>,
        bio: Option<String>,
    ) -> User {
        User {
            id,
            username,
            email,
            avatar,
            bio,
        }
    }
    pub fn mapper_follow(follower: User, following: User) -> Result<MapFollow> {
        Ok(MapFollow {
            follower,
            following,
        })
    }
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
