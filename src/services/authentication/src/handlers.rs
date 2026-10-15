use crate::jwt::generate_jwt;
use crate::models::*;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use mongodb::bson::doc;
use serde_json::json;

pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    if let Ok(Some(_)) = collection
        .find_one(doc! { "username": &req.username })
        .await
    {
        return HttpResponse::Conflict().json(json!({ "error": "Username already exists" }));
    }

    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let hashed_password = match argon2.hash_password(req.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to hash password" }))
        }
    };

    let new_user = User {
        id: mongodb::bson::oid::ObjectId::new(),
        username: req.username.clone(),
        email: "".to_string(),
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

    if let Err(err) = collection.insert_one(new_user).await {
        return HttpResponse::InternalServerError().json(json!({ "error": err.to_string() }));
    }

    HttpResponse::Created().json(json!({ "message": "User registered successfully" }))
}

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    let user = match collection
        .find_one(doc! { "username": &body.username })
        .await
    {
        Ok(Some(user)) => user,
        _ => {
            return HttpResponse::Unauthorized()
                .json(json!({ "message": "Invalid username or password" }));
        }
    };

    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "message": "Invalid password hash" }))
        }
    };

    let is_valid = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        match generate_jwt(&user.id.to_hex(), Some("user"), &state.jwt_secret) {
            Ok(token) => HttpResponse::Created().json(json!({
                "message": "Login successful.",
                "access_token": token,
                "user": User::to_user(user)
            })),
            Err(_) => HttpResponse::InternalServerError().json(json!({
                "message": "An error occurred while generating the access token."
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({
            "message": "Invalid credentials provided. Please check your username and password."
        }))
    }
}
