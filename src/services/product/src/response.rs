use serde::Serialize;

// Success response structure
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

// Error response structure
#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
}