
use serde::Serialize;

// Error response structure
#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
}