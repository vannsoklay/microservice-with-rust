pub mod rate_limit;
pub mod tls;

use super::validate_token;
use actix_service::ServiceExt;
use actix_web::{Error, HttpRequest, HttpResponse, Result};
use jsonwebtoken::errors::Error as JwtError;

pub async fn jwt_auth_middleware(req: HttpRequest, secret: &str) -> Result<HttpResponse, Error> {
    // Retrieve the token from the Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(token) = auth_header.to_str() {
            println!("auth_header {}", token);
            // Validate the token
            match validate_token(token.trim_start_matches("Bearer "), secret) {
                Ok(t) => Ok(HttpResponse::Ok().json(t.claims)),
                Err(_) => Ok(HttpResponse::Unauthorized().finish()),
            }
        } else {
            Ok(HttpResponse::Unauthorized().finish())
        }
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
