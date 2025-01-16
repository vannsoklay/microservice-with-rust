use crate::{db::DBConfig, identify::identify, models::*};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;
use serde_json::json; // Add this import for json!
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, 
        PasswordHasher, 
        PasswordVerifier, 
        SaltString
    },
    Argon2
};
use crate::AppState;

// get all user
pub async fn get_user(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    println!("user_id {}", user_id);

    // Find the user by ID
    let user = match collection.find_one(doc! { "_id": user_id }).await {
        Ok(Some(user)) => user,
        _ => return HttpResponse::NotFound().json(json!({ "error": "User not found" })),
    };

    let response = User::to_user(user);
    HttpResponse::Ok().json(response)
}

// register
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    // Check if username already exists
    if let Ok(Some(_)) = collection.find_one(doc! { "username": &req.username }).await {
        return HttpResponse::Conflict().json(json!({ "error": "Username already exists" }));
    }

    // Generate a random salt
    let salt = argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

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
    req: web::Json<LoginRequest>,
) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    // Find the user by username
    let user = match collection
        .find_one(doc! { "username": &req.username })
        .await
    {
        Ok(Some(user)) => user,
        _ => {
            return HttpResponse::Unauthorized().json(json!({ "error": "Invalid username or password" }));
        }
    };

   // Verify the password
   let parsed_hash = match argon2::password_hash::PasswordHash::new(&user.password) {
    Ok(hash) => hash,
    Err(_) => {
        return HttpResponse::InternalServerError()
            .json(json!({ "error": "Invalid password hash" }))
    }
};

    let is_valid = Argon2::default()
    .verify_password(req.password.as_bytes(), &parsed_hash)
    .is_ok();

    if is_valid {
    HttpResponse::Ok().json(json!({ "message": "Login successful", "user_id": user.id }))
    } else {
        HttpResponse::Unauthorized().json(json!({ "error": "Invalid username or password" }))
    }
}

// change password
pub async fn change_password(
    body: web::Json<ChangePasswordRequest>,
    req: HttpRequest
) -> impl Responder {
    let collection = DBConfig::user_collection().await;

    // Parse the User ID
    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    // Find the user by ID
    let mut user = match collection.find_one(doc! { "_id": user_id.clone() }).await {
        Ok(Some(user)) => user,
        _ => return HttpResponse::NotFound().json(json!({ "error": "User not found" })),
    };

    // Verify the old password
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Invalid password hash" }))
        }
    };

    let is_old_password_valid = Argon2::default()
        .verify_password(body.old_password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_old_password_valid {
        return HttpResponse::Unauthorized().json(json!({ "error": "Invalid old password" }));
    }

    // Generate a new salt and hash the new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let new_password_hash = match argon2.hash_password(body.new_password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to hash new password" }))
        }
    };

    // Update the user's password in the database
    if let Err(err) = collection
        .update_one(
            doc! { "_id": user_id.clone() },
            doc! { "$set": { "password": new_password_hash } }
        )
        .await
    {
        return HttpResponse::InternalServerError().json(json!({ "error": err.to_string() }));
    }

    HttpResponse::Ok().json(json!({ "message": "Password changed successfully" }))
}
