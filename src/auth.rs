use actix_web::{cookie::Cookie, dev::ServiceRequest, HttpRequest, HttpResponse};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn verify_jwt_from_header(
    req: &ServiceRequest,
    secret: &str,
) -> Result<Claims, actix_web::HttpResponse> {
    let token = match req.headers().get("Authorization") {
        Some(header_value) => {
            let auth_str = match header_value.to_str() {
                Ok(s) => s,
                Err(_) => {
                    return Err(actix_web::HttpResponse::Unauthorized().json(json!({
                        "error": "Invalid Authorization header format"
                    })));
                }
            };

            if auth_str.starts_with("Bearer ") {
                auth_str.trim_start_matches("Bearer ").to_string()
            } else {
                return Err(actix_web::HttpResponse::Unauthorized().json(json!({
                    "error": "Expected Bearer token"
                })));
            }
        }
        None => {
            return Err(actix_web::HttpResponse::Unauthorized().json(json!({
                "error": "Missing Authorization header"
            })));
        }
    };

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => Ok(data.claims),
        Err(_) => Err(actix_web::HttpResponse::Unauthorized().json(json!({
            "error": "Invalid or expired token"
        }))),
    }
}
