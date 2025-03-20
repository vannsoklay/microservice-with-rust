use crate::{
    identify::identify,
    models::product::{Product, RequestProduct},
    AppState,
};
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

pub async fn get_all_products(
    state: web::Data<AppState>,
    query: web::Query<ParamQuery>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.product_config_db.clone();
    let param = query.into_inner();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let cursor = collection
        .find(doc! { "owner_id": user_id.clone(), "created_at": { "$lt": DateTime::now()} })
        .skip(param.skip)
        .limit(param.limit)
        .await;

    match cursor {
        Ok(mut cursor) => {
            let mut products = vec![];
            while let Some(product) = cursor.try_next().await.unwrap_or(None) {
                products.push(product);
            }
            let product_count = collection
                .count_documents(
                    doc! { "owner_id": user_id.clone(), "created_at": { "$lt": DateTime::now()} },
                )
                .await
                .unwrap_or(0);
            HttpResponse::Ok().json(json!({
                "data": products,
                "total": product_count,
                "page": param.skip
            }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_product_by_id(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.product_config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let product = collection
        .find_one(doc! { "_id": product_id.into_inner().to_string(), "owner_id": user_id })
        .await;

    match product {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "product not found"})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"error": err.to_string()})),
    }
}

pub async fn create_product(
    state: web::Data<AppState>,
    product: web::Json<RequestProduct>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.product_config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let new_product = Product::new(product.into_inner(), user_id);

    let result = collection.insert_one(new_product).await;

    match result {
        Ok(insert_result) => HttpResponse::Created().json(insert_result.inserted_id),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_product(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
    updated_product: web::Json<RequestProduct>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.product_config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let id = match ObjectId::parse_str(&*product_id) {
        Ok(parsed_id) => parsed_id.to_string(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid product ID"),
    };

    let mut updated_product = updated_product.into_inner();
    updated_product.owner_id = user_id.clone();

    let update_doc = match to_document(&updated_product) {
        Ok(mut doc) => {
            doc.remove("_id");
            doc.remove("permalink");
            doc.remove("created_at");
            doc
        }
        Err(err) => {
            eprintln!("Failed to serialize updated product: {:?}", err);
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
                HttpResponse::NotFound().body("product not found")
            }
        }
        Err(err) => {
            eprintln!("MongoDB update error: {:?}", err);
            HttpResponse::BadRequest().body("Failed to update product")
        }
    }
}

pub async fn delete_product(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let id = match ObjectId::parse_str(&*product_id) {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid product ID",
                "message": "The provided product ID is not a valid ObjectId"
            }))
        }
    };

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(error_response) => return error_response,
    };

    let collection = state.product_config_db.clone();

    match collection
        .update_one(doc! {
            "_id": id.to_string(),
            "owner_id": user_id
        }, doc! { "$set": { "created_at": None::<Option<String>>, "updated_at": None::<Option<String>>, "deleted_at": chrono::Utc::now().to_rfc3339() } })
        .await
    {
        Ok(delete_result) => match delete_result.matched_count {
            1 => HttpResponse::Ok().json(json!({
                "message": "product successfully deleted",
                "product_id": product_id.to_string()
            })),
            0 => HttpResponse::NotFound().json(json!({
                "error": "Not Found",
                "message": "product not found or you do not have permission to delete it"
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": "Unexpected Delete Result",
                "message": "Multiple documents were unexpectedly deleted"
            })),
        },
        Err(db_error) => {
            eprintln!("Database error during product deletion: {:?}", db_error);
            HttpResponse::InternalServerError().json(json!({
                "error": "Database Operation Failed",
                "message": "An error occurred while attempting to delete the product"
            }))
        }
    }
}
