use actix_web::{web, HttpRequest, HttpResponse, Responder};
use reqwest::{header::HeaderMap, Client, Method};
use serde_json::json;
use std::sync::Arc;
use super::ServiceState;

pub async fn proxy_request(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<Arc<ServiceState>>,
) -> impl Responder {
    // Extract the service name from the URL path
    let path: &str = req.path();
    let service_name = if path.starts_with("/accommodation") {
        "accommodation"
    } else if path.starts_with("/user") {
        "user"
    } else if path.starts_with("/order") {
        "order"
    } else {
        // Return 404 if no matching service is found
        return HttpResponse::NotFound().json(json!({
            "error": "Service not found",
        }));
    };

    // Get the next backend for the selected service
    let backend_url = match state.get_next_backend(service_name) {
        Some(url) => url,
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "error": "Service backend not found",
            }));
        }
    };

    // Extract the query string from the request
    let query_string = req.query_string();
    let uri = if query_string.is_empty() {
        format!("{}{}", backend_url, path)
    } else {
        format!("{}{}?{}", backend_url, path, query_string)
    };

    // Create a Reqwest client
    let client = Client::new();

    // Match the request method
    let response = match *req.method() {
        Method::GET => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert("X-User-ID", format!("675871badb5d7bdf5a8ba050").parse().unwrap());
            headers.insert("X-User-Role", "user".parse().unwrap());
            client.get(&uri).headers(headers).send().await
        }
        Method::POST => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert("X-User-ID", format!("675871badb5d7bdf5a8ba050").parse().unwrap());
            headers.insert("X-User-Role", "user".parse().unwrap());
            client.post(&uri).headers(headers).body(body).send().await
        }
        Method::PUT => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert("X-User-ID", format!("675871badb5d7bdf5a8ba050").parse().unwrap());
            headers.insert("X-User-Role", "user".parse().unwrap());
            client.put(&uri).headers(headers).body(body).send().await
        }
        Method::DELETE => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert("X-User-ID", format!("675871badb5d7bdf5a8ba050").parse().unwrap());
            headers.insert("X-User-Role", "user".parse().unwrap());
            client.delete(&uri).headers(headers).body(body).send().await
        }
        _ => {
            return HttpResponse::MethodNotAllowed().json(json!({
                "error": "Only GET, POST, and PUT methods are supported",
            }));
        }
    };

    // Process the backend response
    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.json::<serde_json::Value>().await.unwrap_or_else(|_| {
                json!({ "error": "Error reading response from backend" })
            });
            HttpResponse::build(status).json(body)
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Error: {}", err),
        })),
    }
}
