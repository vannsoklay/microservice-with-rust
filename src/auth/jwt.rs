use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // Subject (user identifier)
    exp: usize,  // Expiration time
}

use jsonwebtoken::errors::Error as JwtError;


pub fn create_token(user_id: &str, secret: &str) -> Result<String, JwtError> {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as usize, // 1 hour expiration
    };

    // Use `?` to propagate any error that `encode` might return
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
pub fn validate_token(
    token: &str,
    secret: &str,
) -> Result<TokenData<Claims>, JwtError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
}
