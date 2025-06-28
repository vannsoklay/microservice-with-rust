use crate::{store::Store, AppState};
use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Local, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateStore {
    pub user_id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub logo_url: Option<String>,
    pub banner_urls: Vec<String>,
    pub phone: Option<String>,
    pub social_links: Vec<String>,
}

pub async fn create_store(
    state: web::Data<AppState>,
    info: web::Json<CreateStore>,
) -> impl Responder {
    let pool = state.db.clone();
    let now_utc = Utc::now();
    
    let store = sqlx::query_as!(
        Store,
        r#"
        INSERT INTO stores (user_id, name, slug, description, logo_url, banner_urls, phone, social_links, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $9)
        RETURNING id, user_id, name, slug, description, logo_url, banner_urls, phone, social_links, is_active, created_at, updated_at, deleted_at
        "#,
        info.user_id,
        info.name,
        info.slug,
        info.description,
        info.logo_url,
        &info.banner_urls,
        info.phone,
        &info.social_links,
        now_utc,
    )
    .fetch_one(&pool)
    .await;

    match store {
        Ok(store) => HttpResponse::Created().json(store),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
    }
}

pub async fn get_store(state: web::Data<AppState>, store_id: web::Path<i32>) -> impl Responder {
    let pool = state.db.clone();
    let store = sqlx::query_as!(
        Store,
        r#"
        SELECT id, user_id, name, slug, description, logo_url, banner_urls, phone, social_links, is_active, created_at, updated_at, deleted_at
        FROM stores WHERE id = $1 AND deleted_at IS NULL
        "#,
        *store_id
    )
    .fetch_optional(&pool)
    .await;

    match store {
        Ok(Some(store)) => HttpResponse::Ok().json(store),
        Ok(None) => HttpResponse::NotFound().body("Store not found"),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
    }
}

// #[derive(Debug, Deserialize)]
// pub struct UpdateStore {
//     pub name: Option<String>,
//     pub slug: Option<String>,
//     pub description: Option<String>,
//     pub logo_url: Option<String>,
//     pub banner_urls: Option<Vec<String>>,
//     pub phone: Option<String>,
//     pub social_links: Option<Vec<String>>,
//     pub is_active: Option<bool>,
// }

// pub async fn update_store(
//     state: web::Data<AppState>,
//     store_id: web::Path<i32>,
//     info: web::Json<UpdateStore>,
// ) -> impl Responder {
//     let pool = state.db.clone();
//     let now = Utc::now();

//     let store = sqlx::query_as!(
//         Store,
//         r#"
//         UPDATE stores SET
//             name = COALESCE($2, name),
//             slug = COALESCE($3, slug),
//             description = COALESCE($4, description),
//             logo_url = COALESCE($5, logo_url),
//             banner_urls = COALESCE($6, banner_urls),
//             phone = COALESCE($7, phone),
//             social_links = COALESCE($8, social_links),
//             is_active = COALESCE($9, is_active),
//             updated_at = $10
//         WHERE id = $1 AND deleted_at IS NULL
//         RETURNING id, user_id, name, slug, description, logo_url, banner_urls, phone, social_links, is_active, created_at, updated_at, deleted_at
//         "#,
//         *store_id,
//         info.name.as_ref(),
//         info.slug.as_ref(),
//         info.description.as_ref(),
//         info.logo_url.as_ref(),
//         info.banner_urls.as_ref(),
//         info.phone.as_ref(),
//         info.social_links.as_ref(),
//         info.is_active,
//         now,
//     )
//     .fetch_optional(&pool)
//     .await;

//     match store {
//         Ok(Some(store)) => HttpResponse::Ok().json(store),
//         Ok(None) => HttpResponse::NotFound().body("Store not found"),
//         Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
//     }
// }

// // Soft-delete (set deleted_at)
// pub async fn delete_store(state: web::Data<AppState>, store_id: web::Path<i32>) -> impl Responder {
//     let pool = state.db.clone();
//     let now = Utc::now();

//     let store = sqlx::query_as!(
//         Store,
//         r#"
//         UPDATE stores SET deleted_at = $2
//         WHERE id = $1 AND deleted_at IS NULL
//         RETURNING id, user_id, name, slug, description, logo_url, banner_urls, phone, social_links, is_active, created_at, updated_at, deleted_at
//         "#,
//         *store_id,
//         now
//     )
//     .fetch_optional(&pool)
//     .await;

//     match store {
//         Ok(Some(store)) => HttpResponse::Ok().json(store),
//         Ok(None) => HttpResponse::NotFound().body("Store not found"),
//         Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
//     }
// }
