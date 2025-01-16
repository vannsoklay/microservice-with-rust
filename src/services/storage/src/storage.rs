use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::fs;
use std::path::Path;

use crate::utils::compress_and_save_image;

// LOCAL STORAGE
pub struct LocalStorageService {
    pub base_path: String, // Directory to store files
}

impl LocalStorageService {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }
}

// S3 STORAGE
pub struct S3StorageService {
    bucket_name: String,
    client: Client,
}

impl S3StorageService {
    pub fn new(bucket_name: String, client: Client) -> Self {
        Self {
            bucket_name,
            client,
        }
    }
}

#[async_trait]
pub trait StorageService {
    async fn upload_file(&self, file_name: &str, file_extension: &str, file_data: &[u8]) -> Result<String, String>;
    async fn delete_file(&self, file_url: &str) -> Result<(), String>;
}

#[async_trait]
impl StorageService for LocalStorageService {
    async fn upload_file(&self, file_name: &str, file_extension: &str, file_data: &[u8]) -> Result<String, String> {
        // Define the file path
        let file_path = Path::new(&self.base_path).join(&file_name);

        // Check if the file is an image
        let is_image = match file_extension.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => true,
            _ => false,
        };

        if is_image {
            // Compress and save the image
            if let Err(err) = compress_and_save_image(file_data, &file_path) {
                return Err(format!("Failed to compress and save image: {}", err));
            }
        } else {
            // Save the file as is
            if let Err(err) = fs::write(&file_path, file_data) {
                return Err(format!("Failed to save file: {}", err));
            }
        }

        // Return the file path as the URL
        Ok(format!("{}/{}", self.base_path, file_name))
    }

    async fn delete_file(&self, file_url: &str) -> Result<(), String> {
        if let Err(err) = fs::remove_file(file_url) {
            return Err(format!("Failed to delete file: {}", err));
        }
        Ok(())
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    async fn upload_file(&self, file_name: &str, _: &str, file_data: &[u8]) -> Result<String, String> {
        let byte_stream = ByteStream::from(file_data.to_vec());
        let response = self
            .client
            .put_object()
            .bucket(&self.bucket_name)
            .key(file_name)
            .body(byte_stream)
            .send()
            .await;

        match response {
            Ok(_) => Ok(format!(
                "https://{}.s3.amazonaws.com/{}",
                self.bucket_name, file_name
            )),
            Err(err) => Err(format!("Failed to upload file: {}", err)),
        }
    }

    async fn delete_file(&self, file_url: &str) -> Result<(), String> {
        // Extract key from file_url and delete
        let key = file_url.split("/").last().unwrap_or_default();
        let result = self
            .client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Failed to delete file: {}", err)),
        }
    }
}
