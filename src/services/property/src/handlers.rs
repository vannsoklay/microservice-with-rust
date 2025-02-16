use crate::{identify::identify, models::Property, AppState};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, to_document, DateTime};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamQuery {
    skip: u64,
    limit: i64,
}

pub async fn get_all_properties(
    state: web::Data<AppState>,
    query: web::Query<ParamQuery>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();
    let param = query.into_inner();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let cursor = collection
        .find(doc! { "user_id": user_id.clone() })
        .skip(param.skip)
        .limit(param.limit)
        .await;

    match cursor {
        Ok(mut cursor) => {
            let mut properties = vec![];
            while let Some(property) = cursor.try_next().await.unwrap_or(None) {
                properties.push(property);
            }
            let property_count = collection
                .count_documents(doc! { "user_id": user_id.clone() })
                .await
                .unwrap_or(0);
            HttpResponse::Ok().json(json!({
                "data": properties,
                "total": property_count,
                "page": param.skip
            }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_property_by_id(
    state: web::Data<AppState>,
    property_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let property = collection
        .find_one(doc! { "_id": property_id.into_inner().to_string(), "user_id": user_id })
        .await;

    match property.unwrap() {
        Some(property) => HttpResponse::Ok().json(property),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_property(
    state: web::Data<AppState>,
    new_perperty: web::Json<Property>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let mut new_perperty = new_perperty.into_inner();
    new_perperty.id = Some(ObjectId::new().to_string());
    new_perperty.owner_id = Some(user_id.clone());
    new_perperty.created_at = Some(DateTime::now().try_to_rfc3339_string().unwrap());
    new_perperty.updated_at = Some(DateTime::now().try_to_rfc3339_string().unwrap());

    let result = collection.insert_one(new_perperty).await;

    match result {
        Ok(insert_result) => HttpResponse::Created().json(insert_result.inserted_id),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_property(
    state: web::Data<AppState>,
    property_id: web::Path<String>,
    updated_property: web::Json<Property>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let id = match ObjectId::parse_str(&*property_id) {
        Ok(parsed_id) => parsed_id.to_string(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid property ID"),
    };

    let mut updated_property = updated_property.into_inner();
    updated_property.owner_id = Some(user_id.clone());
    updated_property.updated_at = Some(DateTime::now().try_to_rfc3339_string().unwrap());

    let update_doc = match to_document(&updated_property) {
        Ok(mut doc) => {
            doc.remove("_id");
            doc.remove("created_at");
            doc
        }
        Err(err) => {
            eprintln!("Failed to serialize updated property: {:?}", err);
            return HttpResponse::BadRequest().body("Failed to process data");
        }
    };

    let result = collection
        .update_one(
            doc! { "_id": id, "owner_id": user_id.clone() },
            doc! { "$set": update_doc },
        )
        .await;

    match result {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                HttpResponse::Ok().json(update_result)
            } else {
                HttpResponse::NotFound().body("Property not found")
            }
        }
        Err(err) => {
            eprintln!("MongoDB update error: {:?}", err);
            HttpResponse::BadRequest().body("Failed to update property")
        }
    }
}

pub async fn delete_property(
    state: web::Data<AppState>,
    property_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let id = match ObjectId::parse_str(&*property_id) {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid property ID",
                "message": "The provided property ID is not a valid ObjectId"
            }))
        }
    };

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(error_response) => return error_response,
    };

    let collection =state.config_db.clone();

    match collection
        .delete_one(doc! {
            "_id": id.to_string(),
            "owner_id": user_id
        })
        .await
    {
        Ok(delete_result) => match delete_result.deleted_count {
            1 => HttpResponse::Ok().json(json!({
                "message": "Property successfully deleted",
                "property_id": property_id.to_string()
            })),
            0 => HttpResponse::NotFound().json(json!({
                "error": "Not Found",
                "message": "Property not found or you do not have permission to delete it"
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": "Unexpected Delete Result",
                "message": "Multiple documents were unexpectedly deleted"
            })),
        },
        Err(db_error) => {
            eprintln!("Database error during property deletion: {:?}", db_error);
            HttpResponse::InternalServerError().json(json!({
                "error": "Database Operation Failed",
                "message": "An error occurred while attempting to delete the property"
            }))
        }
    }
}
