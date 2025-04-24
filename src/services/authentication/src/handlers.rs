use crate::jwt::generate_jwt;
use crate::AppState;
use crate::{db::DBConfig, identify::identify, models::*};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use mongodb::bson::doc;
use serde_json::json;

// register
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    // Check if username already exists
    if let Ok(Some(_)) = collection
        .find_one(doc! { "username": &req.username })
        .await
    {
        return HttpResponse::Conflict().json(json!({ "error": "Username already exists" }));
    }

    // Generate a random salt
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    // Hash the password using Argon2
    let argon2 = Argon2::default();
    let hashed_password = match argon2.hash_password(req.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to hash password" }))
        }
    };

    // Create a new user
    let new_user = User {
        id: mongodb::bson::oid::ObjectId::new(),
        username: req.username.clone(),
        email: "".to_string(), // Add email field to request if needed
        password: hashed_password,
        avatar: None,
        bio: None,
        follower_count: 0,
        following_count: 0,
        is_verified: false,
        last_login: None,
        status: Status::Active,
        created_at: mongodb::bson::DateTime::now(),
        updated_at: mongodb::bson::DateTime::now(),
    };

    // Insert user into the database
    if let Err(err) = collection.insert_one(new_user).await {
        return HttpResponse::InternalServerError().json(json!({ "error": err.to_string() }));
    }

    HttpResponse::Created().json(json!({ "message": "User registered successfully" }))
}

// login
pub async fn login(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let collection = state.db.collection::<User>("users");
    let jwt_secret = match req.headers().get("x-jwt-secret") {
        Some(header_value) => match header_value.to_str() {
            Ok(secret_str) => secret_str.to_string(),
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({
                    "error": "Invalid JWT secret header format."
                }));
            }
        },
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Missing JWT secret in headers."
            }));
        }
    };

    let user = match collection
        .find_one(doc! { "username": &body.username })
        .await
    {
        Ok(Some(user)) => user,
        _ => {
            return HttpResponse::Unauthorized()
                .json(json!({ "error": "Invalid username or password" }));
        }
    };

    let parsed_hash = match argon2::password_hash::PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Invalid password hash" }))
        }
    };

    let is_valid = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        match generate_jwt(&user.id.to_hex(), Some("user"), &jwt_secret) {
            Ok(token) => HttpResponse::Ok().json(json!({
                "message": "Authentication successful.",
                "access_token": token,
                "user_id": user.id
            })),
            Err(_) => HttpResponse::InternalServerError().json(json!({
                "error": "An error occurred while generating the access token."
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({
            "error": "Invalid credentials provided. Please check your username and password."
        }))
    }
}
