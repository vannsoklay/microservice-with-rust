use image::{ImageReader, codecs::jpeg::JpegEncoder};
use std::fs;
use std::path::Path;

pub fn compress_and_save_image(file_data: &[u8], file_path: &Path) -> Result<(), String> {
    // Read the image from the provided data
    let reader = ImageReader::new(std::io::Cursor::new(file_data))
        .with_guessed_format()
        .map_err(|err| format!("Failed to read image: {}", err))?;
    let image = reader
        .decode()
        .map_err(|err| format!("Failed to decode image: {}", err))?;

    // Compress the image (e.g., reduce quality to 80 for JPEG)
    let rgb_image = image.into_rgb8();
    let mut compressed_image_data = Vec::new();

    rgb_image
        .write_with_encoder(JpegEncoder::new_with_quality(
            &mut compressed_image_data,
            80,
        ))
        .map_err(|err| format!("Failed to compress image: {}", err))?;

    // Write the compressed image to disk
    fs::write(file_path, compressed_image_data)
        .map_err(|err| format!("Failed to save image: {}", err))?;

    Ok(())
}
