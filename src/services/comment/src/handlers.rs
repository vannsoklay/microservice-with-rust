use crate::{
    models::{Comment, CommentReq, Params},
    AppState,
};
use actix_web::{web, HttpMessage as _, HttpRequest, HttpResponse, Responder};
use futures::TryStreamExt as _;
use mongodb::bson::{self, doc, Bson, DateTime};

pub async fn create_comment(
    state: web::Data<AppState>,
    body: web::Json<CommentReq>,
    req: HttpRequest,
) -> impl Responder {
    let comment_data = body.into_inner();
    let comment_collection = state.comment_db.clone();
    let post_collection = state.post_db.clone();

    let author_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: missing user ID"
            }));
        }
    };

    let post_filter: bson::Document = doc! { "permalink": &comment_data.permalink.clone(), "author_id": author_id.clone(), "deleted_at": { "$exists": true } };
    match post_collection.find_one(post_filter).await {
        Ok(_) => {
            let new_comment = Comment::insert_body(
                comment_data.permalink.clone(),
                author_id.clone(),
                comment_data.content.clone(),
                comment_data.parent_comment_id.clone(),
            );
            match comment_collection.insert_one(&new_comment).await {
                Ok(_) => HttpResponse::Created().json(serde_json::json!({
                    "message": "Comment created",
                    "data": new_comment,
                })),
                Err(err) => {
                    eprintln!("Failed to insert comment: {:?}", err);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create comment",
                        "details": err.to_string()
                    }))
                }
            }
        }
        Err(err) => {
            eprintln!("Post lookup failed: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to verify post",
                "details": err.to_string()
            }))
        }
    }
}

pub async fn get_comments_by_post(
    state: web::Data<AppState>,
    permalink: web::Path<String>,
    query: web::Query<Params>,
) -> impl Responder {
    let comment_collection = state.comment_db.clone();
    let user_collection = state.user_db.clone();
    let permalink = permalink.into_inner();

    // Pagination defaults
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).min(100);
    let skip = (page - 1) * limit;

    // Sorting config
    let sort_field = query
        .sort_by
        .clone()
        .unwrap_or_else(|| "created_at".to_string());
    let sort_order = query.sort_order.unwrap_or(-1); // default to newest first

    // Only include votes that are not soft-deleted
    let filter = doc! {
        "permalink": &permalink,
        "$or": [
            { "deleted_at": { "$exists": false } },
            { "deleted_at": Bson::Null }
        ]
    };

    let cursor_result = comment_collection
        .find(filter)
        .sort(doc! { &sort_field: sort_order })
        .skip(skip)
        .limit(limit as i64)
        .await;

    match cursor_result {
        Ok(mut cursor) => {
            let mut comments = Vec::new();
            while let Some(comment) = cursor.try_next().await.unwrap_or(None) {
                comments.push(comment);
            }

            let mut results = Vec::new();

            for comment in comments {
                let author = if let Some(author_id) = Some(comment.author_id.to_owned()) {
                    user_collection
                        .find_one(doc! { "_id": author_id })
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

                let comment_json =
                    serde_json::to_value(Comment::to_response(author, comment)).unwrap();

                results.push(comment_json);
            }

            return HttpResponse::Ok().json(serde_json::json!({
                "message": "Comments retrieved successfully",
                "permalink": permalink,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "count": results.len()
                },
                "comments": results
            }));
        }
        Err(err) => {
            eprintln!("Failed to retrieve comments: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve comments",
                "details": err.to_string()
            }))
        }
    }
}

pub async fn update_comment(
    state: web::Data<AppState>,
    comment_id: web::Path<String>,
    body: web::Json<CommentReq>,
    req: HttpRequest,
) -> impl Responder {
    let comment_collection = state.comment_db.clone();
    let comment_id = comment_id.into_inner();
    let update_data = body.into_inner();

    let author_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: missing user ID"
            }));
        }
    };

    let filter = doc! { "_id": comment_id.to_owned(), "author_id": &author_id };

    match comment_collection
        .find_one_and_update(
            filter,
            doc! { "$set": { "content": update_data.content, "updated_at": DateTime::now().try_to_rfc3339_string().unwrap() } },
        )
        .await
    {
        Ok(Some(updated_comment)) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Comment updated successfully",
            "data": updated_comment
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Comment not found or you do not have permission to update it"
        })),
        Err(err) => {
            eprintln!("Failed to update comment: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update comment",
                "details": err.to_string()
            }))
        }
    }
}

pub async fn delete_comment(
    state: web::Data<AppState>,
    comment_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let comment_collection = state.comment_db.clone();
    let comment_id = comment_id.into_inner();

    let author_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: missing user ID"
            }));
        }
    };

    let filter = doc! { "_id": comment_id.to_owned(), "author_id": &author_id };

    match comment_collection
        .find_one_and_update(
            filter,
            doc! { "$set": { "deleted_at": DateTime::now().try_to_rfc3339_string().unwrap() } },
        )
        .await
    {
        Ok(Some(_)) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Comment deleted successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Comment not found or you do not have permission to delete it"
        })),
        Err(err) => {
            eprintln!("Failed to delete comment: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete comment",
                "details": err.to_string()
            }))
        }
    }
}
