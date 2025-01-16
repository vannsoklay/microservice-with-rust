use crate::{identify::identify, models::Accommodation, AppState};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, to_document};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamQuery {
    skip: u64,
    limit: i64,
}

// get all accommodations
pub async fn get_all_accommodations(
    state: web::Data<AppState>,
    query: web::Query<ParamQuery>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.db.collection::<Accommodation>("accommodations");
    let param = query.into_inner();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let cursor = collection
        .find(doc! { "owner_id" : user_id.clone() })
        .skip(param.skip.to_owned())
        .limit(param.limit.to_owned())
        .await;

    match cursor {
        Ok(mut cursor) => {
            let mut accommodations = vec![];
            while let Some(accommodation) = cursor.try_next().await.unwrap_or(None) {
                accommodations.push(accommodation);
            }
            let accoummodation_count = collection
                .count_documents(doc! { "owner_id" : user_id.clone() })
                .await
                .unwrap_or(0);
            HttpResponse::Ok().json(serde_json::json!({
                "data": accommodations,
                "total": accoummodation_count,
                "page": param.skip.to_owned()
            }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// get accommodation by id
pub async fn get_accommodation_by_id(
    state: web::Data<AppState>,
    accommodation_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.db.collection::<Accommodation>("accommodations");

    // Identify the user
    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let accommodation = collection
        .find_one(doc! { "_id": accommodation_id.into_inner().to_string(), "owner_id": user_id })
        .await;

    match accommodation.unwrap() {
        Some(accommodation) => HttpResponse::Ok().json(accommodation),
        None => HttpResponse::NotFound().finish(),
    }
}

// Create a new accommodation
pub async fn create_accommodation(
    state: web::Data<AppState>,
    new_accommodation: web::Json<Accommodation>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.db.collection::<Accommodation>("accommodations");

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    // Clone the incoming data and set additional fields
    let mut new_accommodation = new_accommodation.into_inner();
    new_accommodation.id = ObjectId::new().to_string(); // Generate a new ObjectId
    new_accommodation.owner_id = Some(user_id);
    new_accommodation.created_at = Utc::now().to_rfc3339();
    new_accommodation.updated_at = Utc::now().to_rfc3339();

    let result = collection.insert_one(new_accommodation).await;

    match result {
        Ok(insert_result) => HttpResponse::Created().json(insert_result.inserted_id),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Update an existing accommodation
pub async fn update_accommodation(
    state: web::Data<AppState>,
    accommodation_id: web::Path<String>,
    updated_accommodation: web::Json<Accommodation>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.db.collection::<Accommodation>("accommodations");

    // Identify the user
    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    // Parse the provided ObjectId
    let id = match ObjectId::parse_str(&*accommodation_id) {
        Ok(parsed_id) => parsed_id.to_string(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid accommodation ID"),
    };

    // Prepare the update document
    let mut updated_accommodation = updated_accommodation.into_inner();
    updated_accommodation.owner_id = Some(user_id.clone());
    updated_accommodation.updated_at = Utc::now().to_rfc3339();

    // Serialize the updated data into a BSON document
    let update_doc = match to_document(&updated_accommodation) {
        Ok(mut doc) => {
            doc.remove("_id"); // Explicitly remove `_id` if it exists
            doc
        }
        Err(err) => {
            eprintln!("Failed to serialize updated accommodation: {:?}", err);
            return HttpResponse::BadRequest().body("Failed to process data");
        }
    };

    // Perform the update operation
    let result = collection
        .update_one(
            doc! { "_id": id, "owner_id": user_id.clone() },
            doc! { "$set": update_doc },
        )
        .await;

    // Handle the update result
    match result {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                HttpResponse::Ok().json(update_result)
            } else {
                HttpResponse::NotFound().body("Accommodation not found")
            }
        }
        Err(err) => {
            eprintln!("MongoDB update error: {:?}", err);
            HttpResponse::BadRequest().body("Failed to update accommodation")
        }
    }
}

// delete a accommodation
pub async fn delete_accommodation(
    state: web::Data<AppState>,
    accommodation_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    // Validate and parse the accommodation ID
    let id = match ObjectId::parse_str(&*accommodation_id) {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid accommodation ID",
                "message": "The provided accommodation ID is not a valid ObjectId"
            }))
        }
    };

    // Identify the user
    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(error_response) => return error_response,
    };

    // Prepare the database collection
    let collection = state.db.collection::<Accommodation>("accommodations");

    // Attempt to delete the accommodation
    match collection
        .delete_one(doc! {
            "_id": id.to_string(),
            "owner_id": user_id
        })
        .await
    {
        Ok(delete_result) => match delete_result.deleted_count {
            1 => HttpResponse::Ok().json(json!({
                "message": "Accommodation successfully deleted",
                "accommodation_id": accommodation_id.to_string()
            })),
            0 => HttpResponse::NotFound().json(json!({
                "error": "Not Found",
                "message": "Accommodation not found or you do not have permission to delete it"
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": "Unexpected Delete Result",
                "message": "Multiple documents were unexpectedly deleted"
            })),
        },
        Err(db_error) => {
            // Log the actual database error for server-side tracking
            eprintln!(
                "Database error during accommodation deletion: {:?}",
                db_error
            );
            HttpResponse::InternalServerError().json(json!({
                "error": "Database Operation Failed",
                "message": "An error occurred while attempting to delete the accommodation"
            }))
        }
    }
}
