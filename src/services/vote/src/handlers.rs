use crate::{
    models::{Vote, VoteQuery, VoteReq},
    AppState,
};
use actix_web::{web, HttpMessage as _, HttpRequest, HttpResponse, Responder};
use futures::StreamExt as _;
use mongodb::bson::{self, doc, Bson, DateTime};

pub async fn create_or_remove_vote(
    state: web::Data<AppState>,
    body: web::Json<VoteReq>,
    req: HttpRequest,
) -> impl Responder {
    let vote_data = body.into_inner();
    let vote_collection = state.vote_db.clone();
    let post_collection = state.post_db.clone();

    let author_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: missing user ID"
            }));
        }
    };

    // 1. Check if post exists and is not deleted
    let post_filter = doc! { "permalink": &vote_data.permalink, "deleted_at": { "$exists": true } };
    match post_collection.find_one(post_filter).await {
        Ok(Some(_)) => {
            // 2. Check if vote already exists
            let vote_filter = doc! {
                "permalink": &vote_data.permalink,
                "author_id": &author_id,
            };

            match vote_collection.find_one(vote_filter.clone()).await {
                Ok(Some(existing_vote)) => {
                    // 2a. Existing vote found
                    if existing_vote.deleted_at.is_some() {
                        // Restore vote (was soft-deleted)
                        let update = doc! {
                            "$set": {
                                "vote_type": "up",
                                "updated_at": DateTime::now().try_to_rfc3339_string().unwrap(),
                                "deleted_at": bson::Bson::Null,
                            }
                        };
                        match vote_collection.update_one(vote_filter, update).await {
                            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                                "message": "Vote restored",
                                "permalink": vote_data.permalink,
                            })),
                            Err(err) => {
                                eprintln!("Failed to restore vote: {:?}", err);
                                HttpResponse::InternalServerError().json(serde_json::json!({
                                    "error": "Failed to restore vote",
                                    "details": err.to_string()
                                }))
                            }
                        }
                    } else {
                        // Soft-delete the vote
                        let update = doc! {
                            "$set": {
                                "vote_type": "removed",
                                "updated_at": bson::Bson::Null,
                                "deleted_at": DateTime::now().try_to_rfc3339_string().unwrap(),
                            }
                        };
                        match vote_collection.update_one(vote_filter, update).await {
                            Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                                "message": "Vote removed"
                            })),
                            Err(err) => {
                                eprintln!("Failed to remove vote: {:?}", err);
                                HttpResponse::InternalServerError().json(serde_json::json!({
                                    "error": "Failed to remove vote",
                                    "details": err.to_string()
                                }))
                            }
                        }
                    }
                }
                Ok(None) => {
                    // 2b. No vote yet → create new
                    let new_vote = Vote::insert_body(
                        vote_data.permalink.clone(),
                        author_id,
                        "up".to_string(), // Default to "up" vote
                    );

                    match vote_collection.insert_one(new_vote).await {
                        Ok(_) => HttpResponse::Created().json(serde_json::json!({
                            "message": "Vote created",
                            "permalink": vote_data.permalink,
                        })),
                        Err(err) => {
                            eprintln!("Failed to insert vote: {:?}", err);
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to create vote",
                                "details": err.to_string()
                            }))
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to find existing vote: {:?}", err);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to check vote existence",
                        "details": err.to_string()
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        })),
        Err(err) => {
            eprintln!("Post lookup failed: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to verify post",
                "details": err.to_string()
            }))
        }
    }
}

pub async fn get_votes_by_post(
    state: web::Data<AppState>,
    permalink: web::Path<String>,
    query: web::Query<VoteQuery>,
) -> impl Responder {
    let vote_collection = state.vote_db.clone();
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

    match vote_collection
        .find(filter)
        .sort(doc! { &sort_field: sort_order })
        .skip(skip)
        .limit(limit as i64)
        .await
    {
        Ok(cursor) => {
            let votes: Vec<_> = cursor.filter_map(|doc| async { doc.ok() }).collect().await;

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Votes retrieved successfully",
                "permalink": permalink,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "count": votes.len()
                },
                "votes": votes
            }))
        }
        Err(err) => {
            eprintln!("Failed to retrieve votes: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve votes",
                "details": err.to_string()
            }))
        }
    }
}
