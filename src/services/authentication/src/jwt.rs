use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn generate_jwt(
    user_id: &str,
    role: Option<&str>,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 60)) // 1 hour
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        role: role
            .map(|r| r.to_string())
            .unwrap_or_else(|| "user".to_string()),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
