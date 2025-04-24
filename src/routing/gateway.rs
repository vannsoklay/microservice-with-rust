use actix_web::{web, HttpMessage as _, HttpRequest, HttpResponse, Responder};
use reqwest::header::HeaderMap;
use serde_json::json;
use std::sync::Arc;

use crate::{
    auth::Claims,
    routing::ServiceState,
    utils::{build_uri, detect_service},
};

pub async fn forward_request(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<Arc<ServiceState>>,
) -> impl Responder {
    let claims = req.extensions().get::<Claims>().cloned();
    let path = req.path();

    let service_name = match detect_service(path) {
        Some(svc) => svc,
        None => return HttpResponse::NotFound().json(json!({ "error": "Service not found" })),
    };

    if claims.is_none() && service_name != "auth" {
        return HttpResponse::Unauthorized().json(json!({ "error": "Missing or invalid token" }));
    }

    let backend_url = match state.get_next_backend(service_name) {
        Some(url) => url,
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Backend not available for service: {service_name}")
            }))
        }
    };

    let uri = build_uri(backend_url, path, req.query_string());

    #[cfg(debug_assertions)]
    println!("Forwarding to URI: {}", uri);

    let client = &state.http_client;
    let mut builder = client.request(req.method().clone(), &uri);

    // Forward headers
    let mut headers = HeaderMap::new();
    for (key, value) in req.headers().iter() {
        headers.insert(key.clone(), value.clone());
    }
    headers.insert("Content-Type", "application/json".parse().unwrap());

    if let Some(claims) = claims {
        headers.insert("X-Service-Key", "key_accommodation".parse().unwrap());
        headers.insert("X-User-ID", claims.sub.parse().unwrap());
        headers.insert("X-User-Role", claims.role.parse().unwrap());
    }

    headers.insert("x-jwt-secret", "123456789".parse().unwrap());
    builder = builder.headers(headers);

    let response = builder.body(body).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if content_type.contains("application/json") {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => HttpResponse::build(status).json(json),
                    Err(_) => HttpResponse::InternalServerError().json(json!({
                        "error": "Invalid JSON"
                    })),
                }
            } else {
                let text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "<invalid body>".into());
                HttpResponse::build(status).body(text)
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Gateway error: {}", err)
        })),
    }
}
