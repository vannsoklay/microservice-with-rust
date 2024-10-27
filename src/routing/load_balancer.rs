use actix_web::{web, HttpRequest, HttpResponse, Responder};
use reqwest::{header::HeaderMap, Client, Method};

use std::sync::Arc;

use super::ServiceState;

pub async fn proxy_request(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<Arc<ServiceState>>,
) -> impl Responder {
    // Extract the service name from the URL path (e.g., /product/items, /user/info)
    let path = req.path();
    let service_name = if path.starts_with("/product") {
        "product"
    } else if path.starts_with("/user") {
        "user"
    } else if path.starts_with("/order") {
        "order"
    } else {
        // Return 404 if no matching service is found
        return HttpResponse::NotFound().body("Service not found");
    };
    // Get the next backend for the selected service
    let backend_url = state
        .get_next_backend(service_name)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Service backend not found"))
        .unwrap();

    // Create full URI by combining backend URL and path
    let uri = format!("{}{}", backend_url, path);
    println!("uri {}", uri);
    // Create a Reqwest client
    let client = Client::new();
    // Match the request method
    let response = match req.method() {
        &Method::GET => {
            // Forward the GET request to the backend
            client.get(&uri).send().await
        }
        &Method::POST => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert("X-User-ID", format!("123").parse().unwrap());
            headers.insert("X-User-Role", "admin".parse().unwrap());
            // Forward the POST request with body to the backend
            client.post(&uri).headers(headers).body(body).send().await
        }
        &Method::PUT => {
            // Forward the PUT request with body to the backend
            client.put(&uri).body(body).send().await
        }
        _ => {
            // Handle unsupported HTTP methods
            return HttpResponse::MethodNotAllowed()
                .body("Only GET, POST, and PUT methods are supported");
        }
    };

    // Process the backend response
    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "Error reading response".into());
            HttpResponse::build(status).body(body)
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}
