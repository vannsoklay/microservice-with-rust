use actix_web::{web, HttpResponse, Responder};
use reqwest::Client;

pub async fn get_products(client: web::Data<Client>, service_url: web::Data<String>) -> impl Responder {
    let response = client.get(format!("{}/products", *service_url))
        .send()
        .await;

    match response {
        Ok(res) => {
            let body = res.text().await.unwrap_or_default();
            HttpResponse::Ok().body(body)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn add_product(client: web::Data<Client>, service_url: web::Data<String>, product: web::Json<serde_json::Value>) -> impl Responder {
    let response = client.post(format!("{}/products", *service_url))
        .json(&product.into_inner())
        .send()
        .await;

    match response {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}