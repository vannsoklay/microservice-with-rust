use mongodb::bson::{
    oid::ObjectId,
    serde_helpers::{
        deserialize_bson_datetime_from_rfc3339_string, serialize_bson_datetime_as_rfc3339_string,
        serialize_object_id_as_hex_string,
    },
    DateTime,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub follower_count: i32,
    pub following_count: i32,
    pub is_verified: bool,
    pub last_login: Option<DateTime>,
    pub status: Status,
    #[serde(
        deserialize_with = "deserialize_bson_datetime_from_rfc3339_string",
        serialize_with = "serialize_bson_datetime_as_rfc3339_string"
    )]
    pub created_at: DateTime,
    #[serde(
        deserialize_with = "deserialize_bson_datetime_from_rfc3339_string",
        serialize_with = "serialize_bson_datetime_as_rfc3339_string"
    )]
    pub updated_at: DateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    #[serde(rename(serialize = "id"))]
    #[serde(rename(deserialize = "_id"))]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub follower_count: i32,
    pub following_count: i32,
    pub is_verified: bool,
    pub last_login: Option<DateTime>,
    pub status: Status,
    pub created_at: String,
    pub updated_at: String,
}

// Default values for fields
impl Default for User {
    fn default() -> Self {
        User {
            id: ObjectId::new(),
            username: String::new(),
            email: String::new(),
            password: String::new(),
            avatar: None,
            bio: None,
            follower_count: 0,
            following_count: 0,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            is_verified: false,
            last_login: None,
            status: Status::Active,
        }
    }
}

impl User {
    pub fn to_user(user: User) -> UserResponse {
        UserResponse {
            id: user.id.to_owned(),
            username: user.username.to_owned(),
            email: user.email.to_owned(),
            avatar: user.avatar.to_owned(),
            bio: user.bio.to_owned(),
            follower_count: user.follower_count.to_owned(),
            following_count: user.following_count.to_owned(),
            is_verified: user.is_verified.to_owned(),
            last_login: user.last_login.to_owned(),
            status: user.status.to_owned(),
            created_at: user.created_at.to_owned().to_string(),
            updated_at: user.updated_at.to_owned().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
