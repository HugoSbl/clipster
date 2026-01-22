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
use std::path::{Path, PathBuf};

/// Default thumbnail size (max dimension)
/// Using 400px for sharp previews on retina displays
const THUMBNAIL_MAX_SIZE: u32 = 400;

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

        eprintln!("[DEBUG file_storage.save_image]");
        eprintln!("  id: {}", id);
        eprintln!("  path: {:?}", path);
        eprintln!("  image dimensions: {}x{}", image.width(), image.height());
        eprintln!("  color type: {:?}", image.color());

        image
            .save_with_format(&path, ImageFormat::Png)
            .map_err(|e| format!("Failed to save image: {}", e))?;

        // Verify what was saved
        if let Ok(meta) = std::fs::metadata(&path) {
            eprintln!("  SAVED OK: {} bytes", meta.len());
        }

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

/// Generate a thumbnail with default max size (400px)
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

/// Generate a thumbnail for a file (macOS)
/// Uses Quick Look for documents (PDF, Word, etc.) and the image crate for images
/// Returns PNG bytes on success, None if thumbnail cannot be generated
#[cfg(target_os = "macos")]
pub fn generate_file_thumbnail_macos(path: &Path, max_size: u32) -> Option<Vec<u8>> {
    // Check if file exists and is accessible
    if !path.exists() {
        return None;
    }

    // For image files, use the image crate directly for best quality
    if is_image_file_macos(path) {
        return generate_thumbnail_from_image_file(path, max_size);
    }

    // For documents (PDF, Word, Excel, etc.), use Quick Look via qlmanage
    generate_quicklook_thumbnail(path, max_size)
}

/// Check if a file is an image based on extension (macOS)
#[cfg(target_os = "macos")]
fn is_image_file_macos(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    matches!(
        extension.as_deref(),
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp")
        | Some("webp") | Some("ico") | Some("tiff") | Some("tif") | Some("heic") | Some("heif")
    )
}

/// Load file as image directly using the image crate
#[cfg(target_os = "macos")]
fn generate_thumbnail_from_image_file(path: &Path, max_size: u32) -> Option<Vec<u8>> {
    let image = image::open(path).ok()?;
    generate_thumbnail(&image, max_size).ok()
}

/// Check if Quick Look supports this file type
/// Skip code files and other types that qlmanage hangs on
#[cfg(target_os = "macos")]
fn is_quicklook_supported(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // Skip code files and other types that Quick Look doesn't handle well
    let unsupported = matches!(
        extension.as_deref(),
        Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("rs") | Some("py")
        | Some("rb") | Some("go") | Some("java") | Some("c") | Some("cpp") | Some("h")
        | Some("cs") | Some("php") | Some("swift") | Some("kt") | Some("scala")
        | Some("vue") | Some("svelte") | Some("json") | Some("yaml") | Some("yml")
        | Some("toml") | Some("xml") | Some("csv") | Some("sql") | Some("sh") | Some("bash")
        | Some("zsh") | Some("fish") | Some("ps1") | Some("bat") | Some("cmd")
        | Some("lock") | Some("log") | Some("env") | Some("gitignore") | Some("dockerignore")
    );

    !unsupported
}

/// Generate thumbnail using Quick Look (qlmanage command)
/// Works for PDF, Word, Excel, PowerPoint, Pages, Keynote, Numbers, etc.
#[cfg(target_os = "macos")]
fn generate_quicklook_thumbnail(path: &Path, max_size: u32) -> Option<Vec<u8>> {
    use std::process::{Command, Stdio};
    use std::fs;
    use std::time::Duration;

    // Skip unsupported file types to avoid qlmanage hanging
    if !is_quicklook_supported(path) {
        eprintln!("[generate_quicklook_thumbnail] Skipping unsupported file type: {:?}", path.extension());
        return None;
    }

    // Create a temporary directory for the thumbnail
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("clipster_ql_{}", timestamp));
    fs::create_dir_all(&temp_dir).ok()?;

    // Use qlmanage to generate thumbnail with timeout
    // -t = thumbnail mode
    // -s = size
    // -o = output directory
    let mut child = Command::new("qlmanage")
        .args([
            "-t",
            "-s", &max_size.to_string(),
            "-o", temp_dir.to_str()?,
            path.to_str()?,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    // Wait with timeout (3 seconds max)
    let timeout = Duration::from_secs(3);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break, // Process finished
            Ok(None) => {
                if start.elapsed() > timeout {
                    eprintln!("[generate_quicklook_thumbnail] Timeout - killing qlmanage");
                    let _ = child.kill();
                    let _ = fs::remove_dir_all(&temp_dir);
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                let _ = fs::remove_dir_all(&temp_dir);
                return None;
            }
        }
    }

    // Find the generated thumbnail file
    // qlmanage creates files with .png extension
    let entries = fs::read_dir(&temp_dir).ok()?;
    let thumbnail_path = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().map(|e| e == "png").unwrap_or(false))?;

    // Read the thumbnail PNG
    let png_data = fs::read(&thumbnail_path).ok()?;

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    // Verify it's a valid PNG and not too small (qlmanage might fail silently)
    if png_data.len() < 100 {
        return None;
    }

    Some(png_data)
}

/// Generate a thumbnail for a file on Windows
/// For image files: uses the image crate directly
/// For other files: extracts the file type icon using SHGetFileInfoW
#[cfg(target_os = "windows")]
pub fn generate_file_thumbnail_windows(path: &Path, max_size: u32) -> Option<Vec<u8>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    if !path.exists() {
        return None;
    }

    // For image files, use the image crate directly for best quality
    if is_image_file(path) {
        return generate_thumbnail_from_image_file_windows(path, max_size);
    }

    // For non-image files, extract the file type icon
    extract_file_icon_windows(path, max_size)
}

/// Check if a file is an image based on extension
#[cfg(target_os = "windows")]
fn is_image_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    matches!(
        extension.as_deref(),
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("webp") | Some("ico") | Some("tiff") | Some("tif")
    )
}

/// Generate thumbnail from image file using the image crate
#[cfg(target_os = "windows")]
fn generate_thumbnail_from_image_file_windows(path: &Path, max_size: u32) -> Option<Vec<u8>> {
    let image = image::open(path).ok()?;
    generate_thumbnail(&image, max_size).ok()
}

/// Extract file type icon using Shell API and convert to PNG
#[cfg(target_os = "windows")]
fn extract_file_icon_windows(path: &Path, max_size: u32) -> Option<Vec<u8>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    unsafe {
        // Convert path to wide string
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

        // Get file info with icon
        let mut file_info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || file_info.hIcon.is_invalid() {
            return None;
        }

        // Convert HICON to PNG bytes
        let png_bytes = convert_hicon_to_png(file_info.hIcon, max_size);

        // Clean up the icon
        let _ = DestroyIcon(file_info.hIcon);

        png_bytes
    }
}

/// Convert HICON to PNG bytes
#[cfg(target_os = "windows")]
fn convert_hicon_to_png(hicon: windows::Win32::UI::WindowsAndMessaging::HICON, max_size: u32) -> Option<Vec<u8>> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

    unsafe {
        // Get icon info to access the bitmap
        let mut icon_info = ICONINFO::default();
        if !GetIconInfo(hicon, &mut icon_info).as_bool() {
            return None;
        }

        // Create a device context
        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            if !icon_info.hbmColor.is_invalid() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                DeleteObject(icon_info.hbmMask);
            }
            return None;
        }

        // Use the color bitmap (hbmColor) if available
        let hbitmap = if !icon_info.hbmColor.is_invalid() {
            icon_info.hbmColor
        } else {
            // Fallback to mask bitmap for monochrome icons
            icon_info.hbmMask
        };

        // Select the bitmap into the DC
        let old_bitmap = SelectObject(hdc, hbitmap);

        // Get bitmap dimensions
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 0,
                biHeight: 0,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default(); 1],
        };

        // First call to get dimensions
        if GetDIBits(hdc, hbitmap, 0, 0, None, &mut bmi, DIB_RGB_COLORS) == 0 {
            SelectObject(hdc, old_bitmap);
            DeleteDC(hdc);
            if !icon_info.hbmColor.is_invalid() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                DeleteObject(icon_info.hbmMask);
            }
            return None;
        }

        let width = bmi.bmiHeader.biWidth.unsigned_abs();
        let height = bmi.bmiHeader.biHeight.unsigned_abs();

        if width == 0 || height == 0 {
            SelectObject(hdc, old_bitmap);
            DeleteDC(hdc);
            if !icon_info.hbmColor.is_invalid() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                DeleteObject(icon_info.hbmMask);
            }
            return None;
        }

        // Prepare for pixel extraction
        bmi.bmiHeader.biHeight = -(height as i32); // Top-down DIB
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0;

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Get the actual pixel data
        if GetDIBits(
            hdc,
            hbitmap,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        ) == 0
        {
            SelectObject(hdc, old_bitmap);
            DeleteDC(hdc);
            if !icon_info.hbmColor.is_invalid() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                DeleteObject(icon_info.hbmMask);
            }
            return None;
        }

        // Clean up GDI objects
        SelectObject(hdc, old_bitmap);
        DeleteDC(hdc);
        if !icon_info.hbmColor.is_invalid() {
            DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_invalid() {
            DeleteObject(icon_info.hbmMask);
        }

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap B and R
        }

        // Create image from pixels
        let img = image::RgbaImage::from_raw(width, height, pixels)?;
        let dynamic_img = DynamicImage::ImageRgba8(img);

        // Generate thumbnail at the requested size
        generate_thumbnail(&dynamic_img, max_size).ok()
    }
}

/// Stub for non-macOS and non-Windows platforms - always returns None
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn generate_file_thumbnail_macos(_path: &Path, _max_size: u32) -> Option<Vec<u8>> {
    None
}

/// Stub for non-macOS and non-Windows platforms - always returns None
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn generate_file_thumbnail_windows(_path: &Path, _max_size: u32) -> Option<Vec<u8>> {
    None
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
