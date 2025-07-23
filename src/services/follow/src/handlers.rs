use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use futures::StreamExt as _;
use mongodb::bson::{DateTime, doc, oid::ObjectId};
use serde_json::json;

use crate::{
    AppState,
    models::{Follow, FollowRequest, Query, StatusQuery, User},
};

pub async fn follow(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<FollowRequest>,
) -> impl Responder {
    let follower_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "message": "You must be logged in to follow users."
            }));
        }
    };

    if follower_id == payload.following_id {
        return HttpResponse::BadRequest().json(json!({
            "message": "You cannot follow yourself."
        }));
    }

    let filter = doc! {
        "follower_id": &follower_id,
        "following_id": &payload.following_id
    };

    let exists = state.follow_db.find_one(filter.clone()).await;

    if let Ok(Some(_)) = exists {
        return HttpResponse::Conflict().json(json!({
            "message": "You're already following this user."
        }));
    }

    let follow = Follow {
        id: ObjectId::new(),
        follower_id,
        following_id: payload.following_id.clone(),
        created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
    };

    match state.follow_db.insert_one(follow).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Successfully followed user."
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "message": "Something went wrong. Please try again later."
        })),
    }
}

pub async fn unfollow(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<FollowRequest>,
) -> impl Responder {
    let follower_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "message": "You must be logged in to unfollow users."
            }));
        }
    };

    let filter = doc! {
        "follower_id": &follower_id,
        "following_id": &payload.following_id
    };

    match state.follow_db.delete_one(filter).await {
        Ok(result) if result.deleted_count > 0 => HttpResponse::Ok().json(json!({
            "message": "Successfully unfollowed user."
        })),
        Ok(_) => HttpResponse::NotFound().json(json!({
            "message": "You are not following this user."
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "message": "Something went wrong. Please try again later."
        })),
    }
}

pub async fn follow_toggle(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<FollowRequest>,
) -> impl Responder {
    let follower_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "message": "You must be logged in to perform this action."
            }));
        }
    };

    let filter = doc! {
        "follower_id": &follower_id,
        "following_id": &payload.following_id
    };

    match state.follow_db.find_one(filter.clone()).await {
        Ok(Some(_)) => {
            // Already followed → Unfollow
            match state.follow_db.delete_one(filter).await {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "message": "Unfollowed successfully.",
                    "status": "unfollowed"
                })),
                Err(_) => HttpResponse::InternalServerError().json(json!({
                    "message": "Failed to unfollow user."
                })),
            }
        }
        Ok(None) => {
            // Not following → Follow
            let follow = Follow {
                id: ObjectId::new(),
                follower_id,
                following_id: payload.following_id.clone(),
                created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            };

            match state.follow_db.insert_one(follow).await {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "message": "Followed successfully.",
                    "status": "followed"
                })),
                Err(_) => HttpResponse::InternalServerError().json(json!({
                    "message": "Failed to follow user."
                })),
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "message": "Something went wrong. Please try again later."
        })),
    }
}

pub async fn follow_status(
    state: web::Data<AppState>,
    query: web::Query<StatusQuery>,
) -> impl Responder {
    let filter = doc! {
        "follower_id": &query.follower_id,
        "following_id": &query.following_id,
    };

    let follow_exists = match state.follow_db.find_one(filter).await {
        Ok(result) => result.is_some(),
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Oops! Something went wrong. Please try again later."
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "follower_id": query.follower_id,
        "following_id": query.following_id,
        "is_following": follow_exists
    }))
}

pub async fn followers(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
    query: web::Query<Query>,
) -> impl Responder {
    use mongodb::bson::{doc, oid::ObjectId};

    let follow_db = state.follow_db.clone();
    let user_db = state.user_db.clone();
    let user_id = user_id.into_inner();

    // Pagination defaults
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).min(100);
    let skip = (page - 1) * limit;

    // Sorting config
    let sort_field = query
        .sort_by
        .clone()
        .unwrap_or_else(|| "created_at".to_string());
    let sort_order = query.sort_order.unwrap_or(-1);

    // Step 1: Find all follows where following_id = user_id
    let mut follows_cursor = match follow_db.find(doc! { "following_id": &user_id }).await {
        Ok(cursor) => cursor,
        Err(_) => {
            return HttpResponse::BadGateway().json(json!({
                "message": "Internal server error"
            }));
        }
    };

    // Step 2: Collect all follower_id values
    let mut follower_ids = Vec::new();
    while let Some(result) = follows_cursor.next().await {
        if let Ok(follow) = result {
            if let Ok(follower_oid) = ObjectId::parse_str(&follow.follower_id) {
                follower_ids.push(follower_oid.to_hex());
            }
        }
    }

    // Step 3: Query user collection for all follower_ids
    let filter = doc! { "_id": { "$in": &follower_ids } };
    let users_cursor = match user_db.find(filter).await {
        Ok(cursor) => cursor,
        Err(_) => {
            return HttpResponse::BadGateway().json(json!({
                "message": "Internal server error"
            }));
        }
    };

    let users: Vec<User> = users_cursor
        .filter_map(|doc| async { doc.ok() })
        .collect()
        .await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Followers retrieved successfully",
        "pagination": {
            "page": page,
            "limit": limit,
            "count": users.len()
        },
        "data": users
    }))
}

pub async fn following(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
    query: web::Query<Query>,
) -> impl Responder {
    let follow_db = state.follow_db.clone();
    let user_id: String = user_id.into_inner();

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

    let data = follow_db
        .find(doc! { "follower_id": user_id })
        .limit(limit as i64)
        .skip(skip)
        .sort(doc! { &sort_field: sort_order })
        .await;

    match data {
        Ok(cursor) => {
            let data: Vec<_> = cursor.filter_map(|doc| async { doc.ok() }).collect().await;

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Following retrieved successfully",
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "count": data.len()
                },
                "data": data
            }))
        }
        Err(_) => {
            return HttpResponse::BadGateway().json(json!({
                "messsage": "Internat server error"
            }));
        }
    }
}

pub async fn follow_count(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
) -> impl Responder {
    let follow_db = state.follow_db.clone();
    let user_id: String = user_id.into_inner();

    let data_count = follow_db
        .count_documents(doc! { "follower_id": user_id })
        .await;

    match data_count {
        Ok(data) => HttpResponse::Ok().json(json!({
            "follow_count": data
        })),
        Err(_) => {
            return HttpResponse::BadGateway().json(json!({
                "messsage": "Internat server error"
            }));
        }
    }
}
