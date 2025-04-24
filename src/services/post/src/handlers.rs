use crate::{identify::identify, models::Post, AppState};
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

pub async fn get_all_posts(
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
        .find(doc! { "author": user_id.clone() })
        .sort(doc! { "created_at": -1 })
        .skip(param.skip)
        .limit(param.limit)
        .await;

    match cursor {
        Ok(mut cursor) => {
            let mut posts = vec![];
            while let Some(post) = cursor.try_next().await.unwrap_or(None) {
                posts.push(post);
            }
            let post_count = collection
                .count_documents(doc! { "author": user_id.clone() })
                .await
                .unwrap_or(0);
            HttpResponse::Ok().json(json!({
                "data": posts,
                "total": post_count,
                "page": param.skip
            }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_post_by_id(
    state: web::Data<AppState>,
    post_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();

    let author_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let post = collection
        .find_one(doc! { "_id": post_id.into_inner().to_string(), "author": author_id })
        .await;

    match post {
        Ok(Some(post)) => HttpResponse::Ok().json(post),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Post not found"})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"error": err.to_string()})),
    }
}

pub async fn create_post(
    state: web::Data<AppState>,
    post: web::Json<Post>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let new_post = Post::new(
        post.content.clone(),
        user_id,
        post.post_type.clone(),
        post.title.clone(),
        post.media_urls.clone(),
        post.tags.clone(),
    );

    let result = collection.insert_one(new_post).await;

    match result {
        Ok(insert_result) => HttpResponse::Created().json(insert_result.inserted_id),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_post(
    state: web::Data<AppState>,
    post_id: web::Path<String>,
    updated_post: web::Json<Post>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.config_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let id = match ObjectId::parse_str(&*post_id) {
        Ok(parsed_id) => parsed_id.to_string(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid post ID"),
    };

    let mut updated_post = updated_post.into_inner();
    updated_post.author = Some(user_id.clone());
    updated_post.updated_at = Some(DateTime::now().try_to_rfc3339_string().unwrap());

    let update_doc = match to_document(&updated_post) {
        Ok(mut doc) => {
            doc.remove("_id");
            doc.remove("permalink");
            doc.remove("created_at");
            doc
        }
        Err(err) => {
            eprintln!("Failed to serialize updated post: {:?}", err);
            return HttpResponse::BadRequest().body("Failed to process data");
        }
    };

    let result = collection
        .update_one(
            doc! { "_id": id, "author": user_id.clone() },
            doc! { "$set": update_doc },
        )
        .await;

    match result {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                HttpResponse::Ok().json(update_result)
            } else {
                HttpResponse::NotFound().body("post not found")
            }
        }
        Err(err) => {
            eprintln!("MongoDB update error: {:?}", err);
            HttpResponse::BadRequest().body("Failed to update post")
        }
    }
}

pub async fn delete_post(
    state: web::Data<AppState>,
    post_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let id = match ObjectId::parse_str(&*post_id) {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid post ID",
                "message": "The provided post ID is not a valid ObjectId"
            }))
        }
    };

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(error_response) => return error_response,
    };

    let collection = state.config_db.clone();

    match collection
        .delete_one(doc! {
            "_id": id.to_string(),
            "author": user_id
        })
        .await
    {
        Ok(delete_result) => match delete_result.deleted_count {
            1 => HttpResponse::Ok().json(json!({
                "message": "post successfully deleted",
                "post_id": post_id.to_string()
            })),
            0 => HttpResponse::NotFound().json(json!({
                "error": "Not Found",
                "message": "post not found or you do not have permission to delete it"
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": "Unexpected Delete Result",
                "message": "Multiple documents were unexpectedly deleted"
            })),
        },
        Err(db_error) => {
            eprintln!("Database error during post deletion: {:?}", db_error);
            HttpResponse::InternalServerError().json(json!({
                "error": "Database Operation Failed",
                "message": "An error occurred while attempting to delete the post"
            }))
        }
    }
}
