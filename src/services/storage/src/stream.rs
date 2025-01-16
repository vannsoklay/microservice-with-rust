use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::path::PathBuf;
use mime_guess::from_path;
use std::fs;

pub async fn stream_image(
    req: HttpRequest,
    file_name: web::Path<String>, // Filename passed as a path parameter
) -> Result<HttpResponse> {
    // Define the base directory where your images are stored
    let base_path = "./uploads"; // Adjust the path as needed
    let file_path = PathBuf::from(base_path).join(file_name.as_str());

    // Check if the file exists and is a valid image
    if file_path.exists() && file_path.is_file() {
        // Use Actix's `NamedFile` to serve the file
        let named_file = NamedFile::open(file_path)?;

        // Stream the file with the correct content type
        Ok(named_file.into_response(&req))
    } else {
        // Return a 404 response if the file does not exist
        Ok(HttpResponse::NotFound().body("Image not found"))
    }
}

pub async fn _stream_image_custom(
    file_name: web::Path<String>,
) -> HttpResponse {
    let base_path = "./uploads";
    let file_path = PathBuf::from(base_path).join(file_name.as_str());

    if file_path.exists() && file_path.is_file() {
        let mime_type = from_path(&file_path).first_or_octet_stream();
        match fs::read(&file_path) {
            Ok(data) => HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(data),
            Err(_) => HttpResponse::InternalServerError().body("Error reading the file"),
        }
    } else {
        HttpResponse::NotFound().body("Image not found")
    }
}