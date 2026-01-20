//! File storage for clipboard images
//!
//! Handles saving full-size images to disk and generating thumbnails.
//! Images are stored as PNG files in ~/.clipster/images/

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::codecs::bmp::BmpDecoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

/// Default thumbnail size (max dimension)
const THUMBNAIL_MAX_SIZE: u32 = 200;

/// File storage manager for clipboard images
pub struct FileStorage {
    /// Base directory for image storage
    images_dir: PathBuf,
}

impl FileStorage {
    /// Create a new file storage instance
    pub fn new() -> Result<Self, String> {
        let images_dir = Self::get_images_dir()?;

        // Ensure directory exists
        fs::create_dir_all(&images_dir)
            .map_err(|e| format!("Failed to create images directory: {}", e))?;

        Ok(Self { images_dir })
    }

    /// Get the images directory path
    fn get_images_dir() -> Result<PathBuf, String> {
        let data_dir = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| "Could not determine home directory".to_string())?;

        Ok(data_dir.join(".clipster").join("images"))
    }

    /// Get the full path for an image file
    pub fn get_image_path(&self, id: &str) -> PathBuf {
        self.images_dir.join(format!("{}.png", id))
    }

    /// Save image data to disk as PNG
    /// Returns the file path on success
    pub fn save_image(&self, id: &str, image: &DynamicImage) -> Result<PathBuf, String> {
        let path = self.get_image_path(id);

        image
            .save_with_format(&path, ImageFormat::Png)
            .map_err(|e| format!("Failed to save image: {}", e))?;

        Ok(path)
    }

    /// Save raw PNG bytes to disk
    pub fn save_png_bytes(&self, id: &str, png_data: &[u8]) -> Result<PathBuf, String> {
        let path = self.get_image_path(id);

        fs::write(&path, png_data).map_err(|e| format!("Failed to write image file: {}", e))?;

        Ok(path)
    }

    /// Delete an image file
    pub fn delete_image(&self, id: &str) -> Result<bool, String> {
        let path = self.get_image_path(id);

        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete image: {}", e))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if an image exists
    pub fn image_exists(&self, id: &str) -> bool {
        self.get_image_path(id).exists()
    }

    /// Load an image from disk
    pub fn load_image(&self, id: &str) -> Result<DynamicImage, String> {
        let path = self.get_image_path(id);

        image::open(&path).map_err(|e| format!("Failed to load image: {}", e))
    }

    /// Get total size of all stored images in bytes
    pub fn total_storage_size(&self) -> Result<u64, String> {
        let mut total = 0u64;

        let entries = fs::read_dir(&self.images_dir)
            .map_err(|e| format!("Failed to read images directory: {}", e))?;

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total += metadata.len();
            }
        }

        Ok(total)
    }

    /// Clean up orphaned images (images not in database)
    /// Takes a list of valid image IDs
    pub fn cleanup_orphans(&self, valid_ids: &[String]) -> Result<usize, String> {
        let mut deleted = 0;

        let entries = fs::read_dir(&self.images_dir)
            .map_err(|e| format!("Failed to read images directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(stem) = path.file_stem() {
                let id = stem.to_string_lossy().to_string();
                if !valid_ids.contains(&id) {
                    if fs::remove_file(&path).is_ok() {
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize file storage")
    }
}

/// Decode CF_DIB data (raw DIB without BMP file header) into a DynamicImage
pub fn decode_dib(dib_data: &[u8]) -> Result<DynamicImage, String> {
    // CF_DIB data is raw BITMAPINFO + pixel data, without the 14-byte BMP file header
    // Use BmpDecoder::new_without_file_header for this
    let cursor = Cursor::new(dib_data);

    let decoder = BmpDecoder::new_without_file_header(cursor)
        .map_err(|e| format!("Failed to create BMP decoder: {}", e))?;

    DynamicImage::from_decoder(decoder).map_err(|e| format!("Failed to decode DIB: {}", e))
}

/// Decode a complete BMP file (with header) into a DynamicImage
pub fn decode_bmp(bmp_data: &[u8]) -> Result<DynamicImage, String> {
    image::load_from_memory_with_format(bmp_data, ImageFormat::Bmp)
        .map_err(|e| format!("Failed to decode BMP: {}", e))
}

/// Generate a thumbnail from a DynamicImage
/// Returns PNG bytes
pub fn generate_thumbnail(image: &DynamicImage, max_size: u32) -> Result<Vec<u8>, String> {
    // Calculate new dimensions preserving aspect ratio
    let (width, height) = (image.width(), image.height());
    let (new_width, new_height) = if width > height {
        let ratio = max_size as f32 / width as f32;
        (max_size, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_size as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_size)
    };

    // Resize using Lanczos3 filter for quality
    let thumbnail = image.resize(new_width, new_height, FilterType::Lanczos3);

    // Encode as PNG
    let mut png_bytes = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    Ok(png_bytes)
}

/// Generate a thumbnail with default max size (200px)
pub fn generate_thumbnail_default(image: &DynamicImage) -> Result<Vec<u8>, String> {
    generate_thumbnail(image, THUMBNAIL_MAX_SIZE)
}

/// Convert thumbnail PNG bytes to base64 string for database storage
pub fn thumbnail_to_base64(png_bytes: &[u8]) -> String {
    BASE64.encode(png_bytes)
}

/// Convert base64 string back to PNG bytes
pub fn base64_to_thumbnail(base64_str: &str) -> Result<Vec<u8>, String> {
    BASE64
        .decode(base64_str)
        .map_err(|e| format!("Failed to decode base64: {}", e))
}

/// Process clipboard image: decode DIB, generate thumbnail, save full image
/// Returns (thumbnail_base64, image_path)
pub fn process_clipboard_image(
    id: &str,
    dib_data: &[u8],
    storage: &FileStorage,
) -> Result<(String, PathBuf), String> {
    // Decode DIB data
    let image = decode_dib(dib_data)?;

    // Generate thumbnail
    let thumbnail_bytes = generate_thumbnail_default(&image)?;
    let thumbnail_base64 = thumbnail_to_base64(&thumbnail_bytes);

    // Save full image
    let image_path = storage.save_image(id, &image)?;

    Ok((thumbnail_base64, image_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for x in 0..width {
            for y in 0..height {
                img.put_pixel(x, y, Rgb([255, 0, 0])); // Red
            }
        }
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_generate_thumbnail_landscape() {
        let image = create_test_image(400, 200);
        let thumbnail = generate_thumbnail(&image, 100).unwrap();

        // Decode thumbnail to check dimensions
        let decoded = image::load_from_memory(&thumbnail).unwrap();

        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 50);
    }

    #[test]
    fn test_generate_thumbnail_portrait() {
        let image = create_test_image(200, 400);
        let thumbnail = generate_thumbnail(&image, 100).unwrap();

        let decoded = image::load_from_memory(&thumbnail).unwrap();

        assert_eq!(decoded.width(), 50);
        assert_eq!(decoded.height(), 100);
    }

    #[test]
    fn test_thumbnail_base64_roundtrip() {
        let original = vec![1, 2, 3, 4, 5];
        let base64 = thumbnail_to_base64(&original);
        let decoded = base64_to_thumbnail(&base64).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_file_storage_path() {
        let storage = FileStorage::new().unwrap();
        let path = storage.get_image_path("test-id");
        assert!(path.to_string_lossy().contains("test-id.png"));
    }
}
