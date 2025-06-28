use std::collections::HashSet;

use crate::{
    identify::identify,
    models::{Post, PostWithAuthor},
    AppState,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use futures::StreamExt as _;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, to_document, Bson, DateTime};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamQuery {
    skip: Option<u64>,
    limit: Option<u64>,
    username: Option<String>,
}

pub async fn get_all_posts(
    state: web::Data<AppState>,
    query: web::Query<ParamQuery>,
) -> impl Responder {
    let post_collection = state.post_db.clone();
    let user_collection = state.user_db.clone();
    let vote_collection = state.vote_db.clone();
    let comment_collection = state.comment_db.clone();

    let params = query.into_inner();
    let page = params.skip.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10);
    let skip = (page - 1) * limit;

    // Optional: Get user ID for the given username (to check vote status)
    let user_id_opt = if let Some(username) = &params.username {
        user_collection
            .find_one(doc! { "username": username })
            .await
            .ok()
            .flatten()
            .and_then(|user| Some(user.id))
    } else {
        None
    };

    // Fetch posts
    let cursor_result = post_collection
        .find(doc! {})
        .sort(doc! { "created_at": -1 })
        .skip(skip)
        .limit(limit as i64)
        .await;

    match cursor_result {
        Ok(mut cursor) => {
            let mut posts = Vec::new();
            let mut permalinks = Vec::new();

            while let Some(post) = cursor.try_next().await.unwrap_or(None) {
                permalinks.push(post.permalink.clone().unwrap_or_default());
                posts.push(post);
            }

            // Fetch all votes made by the user across these posts (soft-deleted votes excluded)
            let voted_permalinks: HashSet<String> = if let Some(ref user_id) = user_id_opt {
                let vote_filter = doc! {
                    "author_id": user_id,
                    "permalink": { "$in": &permalinks },
                    "$or": [
                        { "deleted_at": { "$exists": false } },
                        { "deleted_at": Bson::Null }
                    ]
                };

                match vote_collection.find(vote_filter).await {
                    Ok(cursor) => {
                        cursor
                            .filter_map(|res| async { res.ok().map(|v| v.permalink) })
                            .collect()
                            .await
                    }
                    Err(_) => HashSet::new(),
                }
            } else {
                HashSet::new()
            };

            // Collect enriched post data
            let mut results = Vec::new();

            for post in posts {
                let permalink = post.permalink.clone().unwrap_or_default();

                let vote_or_comment_filter = doc! {
                    "permalink": &permalink,
                    "$or": [
                        { "deleted_at": { "$exists": false } },
                        { "deleted_at": Bson::Null }
                    ]
                };

                let total_votes = vote_collection
                    .count_documents(vote_or_comment_filter.clone())
                    .await
                    .unwrap_or(0);

                let total_comments: u64 = comment_collection
                    .count_documents(vote_or_comment_filter.clone())
                    .await
                    .unwrap_or(0);

                let author = if let Some(author_id) = &post.author_id {
                    user_collection
                        .find_one(doc! { "_id": author_id })
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

                let mut post_json =
                    serde_json::to_value(PostWithAuthor { post, author }.to_response()).unwrap();

                post_json["total_votes"] = json!(total_votes);
                post_json["total_comments"] = json!(total_comments);
                post_json["voted_by_user"] = json!(voted_permalinks.contains(&permalink));

                results.push(post_json);
            }

            let post_count = post_collection.count_documents(doc! {}).await.unwrap_or(0);

            HttpResponse::Ok().json(json!({
                "data": results,
                "total": post_count,
                "page": page,
                "limit": limit
            }))
        }
        Err(err) => {
            eprintln!("Error fetching posts: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_all_posts_by_user(
    state: web::Data<AppState>,
    query: web::Query<ParamQuery>,
    req: HttpRequest,
) -> impl Responder {
    let post_collection = state.post_db.clone();
    let user_collection = state.user_db.clone();
    let vote_collection = state.vote_db.clone();

    let author_id = req.extensions().get::<String>().cloned();

    let params = query.into_inner();
    let page = params.skip.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10);
    let skip = (page - 1) * limit;

    let user_id_opt = if let Some(username) = &params.username {
        user_collection
            .find_one(doc! { "username": username })
            .await
            .ok()
            .flatten()
            .and_then(|user| Some(user.id))
    } else {
        None
    };

    // Fetch posts
    let cursor_result = post_collection
        .find(doc! { "author_id": user_id_opt.clone() })
        .sort(doc! { "created_at": -1 })
        .skip(skip)
        .limit(limit as i64)
        .await;

    match cursor_result {
        Ok(mut cursor) => {
            let mut posts = Vec::new();
            let mut permalinks = Vec::new();

            while let Some(post) = cursor.try_next().await.unwrap_or(None) {
                permalinks.push(post.permalink.clone().unwrap_or_default());
                posts.push(post);
            }

            // Fetch all votes made by the user across these posts (soft-deleted votes excluded)
            let voted_permalinks: HashSet<String> = if let Some(ref user_id) = author_id.clone() {
                let vote_filter = doc! {
                    "author_id": user_id,
                    "permalink": { "$in": &permalinks },
                    "$or": [
                        { "deleted_at": { "$exists": false } },
                        { "deleted_at": Bson::Null }
                    ]
                };

                match vote_collection.find(vote_filter).await {
                    Ok(cursor) => {
                        cursor
                            .filter_map(|res| async { res.ok().map(|v| v.permalink) })
                            .collect()
                            .await
                    }
                    Err(_) => HashSet::new(),
                }
            } else {
                HashSet::new()
            };

            // Collect enriched post data
            let mut results = Vec::new();

            for post in posts {
                let permalink = post.permalink.clone().unwrap_or_default();

                let vote_filter = doc! {
                    "permalink": &permalink,
                    "$or": [
                        { "deleted_at": { "$exists": false } },
                        { "deleted_at": Bson::Null }
                    ]
                };

                let total_votes = vote_collection
                    .count_documents(vote_filter)
                    .await
                    .unwrap_or(0);

                let author = if let Some(author_id) = &post.author_id {
                    user_collection
                        .find_one(doc! { "_id": author_id })
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

                let mut post_json =
                    serde_json::to_value(PostWithAuthor { post, author }.to_response()).unwrap();

                post_json["total_votes"] = json!(total_votes);
                post_json["voted_by_user"] = json!(voted_permalinks.contains(&permalink));

                results.push(post_json);
            }

            let post_count = post_collection.count_documents(doc! {}).await.unwrap_or(0);

            HttpResponse::Ok().json(json!({
                "data": results,
                "total": post_count,
                "page": page,
                "limit": limit
            }))
        }
        Err(err) => {
            eprintln!("Error fetching posts: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_post_by_user(
    state: web::Data<AppState>,
    post_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let post_collection = state.post_db.clone();
    let vote_collection = state.vote_db.clone();
    let user_collection = state.user_db.clone();
    let author_id = req.extensions().get::<String>().cloned();

    let post = post_collection
        .find_one(doc! { "_id": post_id.into_inner().to_string(), "author_id": author_id })
        .await;

    match post {
        Ok(Some(post)) => {
            let permalink = post.permalink.clone().unwrap_or_default();

            let vote_filter = doc! {
                "permalink": &permalink,
                "$or": [
                    { "deleted_at": { "$exists": false } },
                    { "deleted_at": Bson::Null }
                ]
            };

            let total_votes = vote_collection
                .count_documents(vote_filter)
                .await
                .unwrap_or(0);

            let author = if let Some(author_id) = &post.author_id {
                user_collection
                    .find_one(doc! { "_id": author_id })
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };

            let mut post_json =
                serde_json::to_value(PostWithAuthor { post, author }.to_response()).unwrap();

            post_json["total_votes"] = json!(total_votes);
            HttpResponse::Ok().json(post_json)
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"message": "Post not found"})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"message": err.to_string()})),
    }
}

pub async fn get_post_by_permalink(
    state: web::Data<AppState>,
    permalink: web::Path<String>,
    query: web::Query<ParamQuery>,
) -> impl Responder {
    let post_collection = state.post_db.clone();
    let vote_collection = state.vote_db.clone();
    let user_collection = state.user_db.clone();

    let params = query.into_inner();

    // Optional: Get user ID for the given username (to check vote status)
    let user_id_opt = if let Some(username) = &params.username {
        user_collection
            .find_one(doc! { "username": username })
            .await
            .ok()
            .flatten()
            .and_then(|user| Some(user.id))
    } else {
        None
    };

    let post = post_collection
        .find_one(doc! { "permalink": permalink.into_inner().to_string()})
        .await;

    match post {
        Ok(Some(post)) => {
            let permalink = post.permalink.clone().unwrap_or_default();

            let vote_filter = doc! {
                "permalink": &permalink,
                "$or": [
                    { "deleted_at": { "$exists": false } },
                    { "deleted_at": Bson::Null }
                ]
            };
            
            let voted_permalinks: HashSet<String> = if let Some(ref user_id) = user_id_opt {
                let vote_filter = doc! {
                    "author_id": user_id,
                    "permalink": { "$in": &vec![permalink.clone()] },
                    "$or": [
                        { "deleted_at": { "$exists": false } },
                        { "deleted_at": Bson::Null }
                    ]
                };

                match vote_collection.find(vote_filter).await {
                    Ok(cursor) => {
                        cursor
                            .filter_map(|res| async { res.ok().map(|v| v.permalink.clone()) })
                            .collect()
                            .await
                    }
                    Err(_) => HashSet::new(),
                }
            } else {
                HashSet::new()
            };


            let total_votes = vote_collection
                .count_documents(vote_filter)
                .await
                .unwrap_or(0);

            let author = if let Some(author_id) = &post.author_id {
                user_collection
                    .find_one(doc! { "_id": author_id })
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };

            let mut post_json =
                serde_json::to_value(PostWithAuthor { post, author }.to_response()).unwrap();

            post_json["total_votes"] = json!(total_votes);
            post_json["voted_by_user"] = json!(voted_permalinks.contains(&permalink.clone()));
            HttpResponse::Ok().json(post_json)
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"message": "Post not found"})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"message": err.to_string()})),
    }
}

pub async fn create_post(
    state: web::Data<AppState>,
    post: web::Json<Post>,
    req: HttpRequest,
) -> impl Responder {
    let collection = state.post_db.clone();
    let author_id = req.extensions().get::<String>().cloned();

    let new_post = Post::new(
        post.content.clone(),
        author_id,
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
    let collection = state.post_db.clone();

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    let id = match ObjectId::parse_str(&*post_id) {
        Ok(parsed_id) => parsed_id.to_string(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid post ID"),
    };

    let mut updated_post = updated_post.into_inner();
    updated_post.author_id = Some(user_id.clone());
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
            doc! { "_id": id, "author_id": user_id.clone() },
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

    let collection = state.post_db.clone();

    match collection
        .delete_one(doc! {
            "_id": id.to_string(),
            "author_id": user_id
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
