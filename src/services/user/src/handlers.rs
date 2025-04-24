use crate::AppState;
use crate::{db::DBConfig, models::*};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use mongodb::bson::doc;
use serde_json::json;

// get user
pub async fn get_user(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let collection = state.db.collection::<User>("users");

    let user_id = req.extensions().get::<String>().cloned();

    let user = match collection.find_one(doc! { "_id": user_id }).await {
        Ok(Some(user)) => user,
        _ => return HttpResponse::NotFound().json(json!({ "error": "User not found" })),
    };

    let response = User::to_user(user);
    HttpResponse::Ok().json(response)
}

// change password
pub async fn change_password(
    body: web::Json<ChangePasswordRequest>,
    req: HttpRequest,
) -> impl Responder {
    let collection = DBConfig::user_collection().await;

    let user_id = req.extensions().get::<String>().cloned();

    let user = match collection.find_one(doc! { "_id": user_id.clone() }).await {
        Ok(Some(user)) => user,
        _ => return HttpResponse::NotFound().json(json!({ "error": "User not found" })),
    };

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
            doc! { "$set": { "password": new_password_hash } },
        )
        .await
    {
        return HttpResponse::InternalServerError().json(json!({ "error": err.to_string() }));
    }

    HttpResponse::Ok().json(json!({ "message": "Password changed successfully" }))
}
