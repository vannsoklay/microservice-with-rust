use actix_web::{HttpRequest, HttpResponse};

use crate::response::ErrorResponse;

pub async fn identify(req: HttpRequest) -> Result<String, HttpResponse> {
    // Check for user ID and role set by API Gateway
    if let Some(user_id) = req.headers().get("X-User-ID") {
        if let Some(role) = req.headers().get("X-User-Role") {
            if role == "user" {
                let user_id_str = user_id.to_str().unwrap_or("unknown");
                println!("user_id_str {}", user_id_str);
                return Ok(user_id_str.to_string());
            } else {
                return Err(HttpResponse::Forbidden().json(ErrorResponse {
                    success: false,
                    message: "Permission denied".to_string(),
                }));
            }
        }
    }

    Err(HttpResponse::Unauthorized().json(ErrorResponse {
        success: false,
        message: "Unauthorized access".to_string(),
    }))
}
