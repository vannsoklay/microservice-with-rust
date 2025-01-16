use std::path::Path;

use crate::{model::FileMetadata, storage::StorageService, AppState};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use uuid::Uuid;

pub async fn upload_file(app_state: web::Data<AppState>, mut payload: Multipart) -> HttpResponse {
    let storage_service = &app_state.local_storage_service;

    while let Some(field) = payload.next().await {
        if let Ok(mut field) = field {
            let content_disposition = field.content_disposition().unwrap();
            let file_name = content_disposition.get_filename().unwrap().to_string();

            let mut file_data = Vec::new();
            while let Some(chunk) = field.next().await {
                file_data.extend_from_slice(&chunk.unwrap());
            }

            // Generate a unique file name
            let file_extension = Path::new(&file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown");
            let rename_file = format!("{}.{}", Uuid::new_v4(), file_extension);

            // Upload file to storage
            match storage_service.upload_file(&rename_file, file_extension, &file_data).await {
                Ok(file) => {
                    let url = format!("{}/images/{}", String::from("http://127.0.0.1:9000"), rename_file.clone());
                    // Save file metadata to MongoDB
                    let metadata = FileMetadata {
                        id: None,
                        name: rename_file.clone(),
                        url: url.clone(),
                        size: file_data.len() as u64,
                        content_type: field
                            .content_type()
                            .map(|ct| ct.to_string())
                            .unwrap_or_default(),
                        location: file.clone(), // Save location
                    };

                    let collection = app_state.db_config.storage_repo.get_collection();
                    match collection.insert_one(metadata).await {
                        Ok(_) => {
                            return HttpResponse::Ok().json(serde_json::json!({
                                "file_url": url.clone(),
                                "location": file,
                            }));
                        }
                        Err(err) => {
                            return HttpResponse::InternalServerError().body(err.to_string())
                        }
                    }
                }
                Err(err) => return HttpResponse::InternalServerError().body(err),
            }
        }
    }

    HttpResponse::BadRequest().body("No file uploaded")
}

pub async fn delete_file(
    app_state: web::Data<AppState>,
    file_id: web::Path<String>,
) -> HttpResponse {
    let collection = app_state.db_config.storage_repo.get_collection();

    // Parse file ID and delete metadata
    if let Ok(object_id) = ObjectId::parse_str(&*file_id) {
        match collection
            .find_one_and_delete(doc! { "_id": object_id })
            .await
        {
            Ok(Some(metadata)) => {
                // Delete the actual file from storage
                if let Err(err) = app_state
                    .local_storage_service
                    .delete_file(&metadata.url)
                    .await
                {
                    return HttpResponse::InternalServerError().body(err);
                }
                HttpResponse::Ok().body("File deleted successfully")
            }
            Ok(None) => HttpResponse::NotFound().body("File not found"),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid file ID")
    }
}
