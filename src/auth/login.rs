use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use super::jwt::create_token;

// Struct for handling incoming login requests
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

// Struct for JWT claims
#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[post("/api/login")]
pub async fn login(json: web::Json<LoginRequest>) -> impl Responder {
    // Simple user validation logic (replace with real logic)
    let valid_username = "testuser";
    let valid_password = "password123";

    if json.username == valid_username && json.password == valid_password {
        // Create JWT claims with expiration (1 hour from now)
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(1))
            .expect("valid timestamp")
            .timestamp() as usize;
        
        let claims = Claims {
            sub: json.username.clone(),
            exp: expiration,
        };

        let secret = "mykeyismeuknow";
        match create_token(&claims.sub, secret) {
            Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
            Err(_) => HttpResponse::InternalServerError().body("Token generation failed"),
        }
    } else {
        HttpResponse::Unauthorized().body("Invalid username or password")
    }
}