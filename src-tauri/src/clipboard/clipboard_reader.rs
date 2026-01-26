//! Cross-platform clipboard reader
//!
//! Provides unified API for reading clipboard content on Windows and macOS.
//! Supports: Text, Images, and Files

use crate::models::ContentType;

/// Result of reading clipboard content
#[derive(Debug)]
pub enum ClipboardContent {
    Text(String),
    Image(ImageData),
    Files(Vec<String>),
    Empty,
}

/// Image data from clipboard
#[derive(Debug)]
pub struct ImageData {
    /// PNG-encoded image data
    pub png_data: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

// ============================================================================
// Windows Implementation
// ============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use clipboard_win::{formats, get_clipboard, is_format_avail, raw::is_format_avail as is_raw_avail};
    use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, OpenClipboard,
    };
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

    /// Windows clipboard format constants
    pub mod clipboard_formats {
        pub const CF_TEXT: u32 = 1;
        pub const CF_BITMAP: u32 = 2;
        pub const CF_DIB: u32 = 8;
        pub const CF_UNICODETEXT: u32 = 13;
        pub const CF_HDROP: u32 = 15;
        pub const CF_DIBV5: u32 = 17;
    }

    /// Detect the primary content type available on the clipboard
    pub fn detect_format() -> ContentType {
        if is_format_avail(formats::CF_UNICODETEXT) {
            return ContentType::Text;
        }
        if is_raw_avail(clipboard_formats::CF_DIB) || is_raw_avail(clipboard_formats::CF_DIBV5) {
            return ContentType::Image;
        }
        if is_format_avail(formats::CF_HDROP) {
            return ContentType::Files;
        }
        ContentType::Text
    }

    /// Read text from clipboard
    pub fn read_text() -> Option<String> {
        match get_clipboard::<String, _>(formats::Unicode) {
            Ok(text) if !text.is_empty() => Some(text),
            _ => None,
        }
    }

    /// Read image data from clipboard (CF_DIB)
    pub fn read_image() -> Option<ImageData> {
        if let Some(data) = read_dib_format(clipboard_formats::CF_DIB) {
            return Some(data);
        }
        if let Some(data) = read_dib_format(clipboard_formats::CF_DIBV5) {
            return Some(data);
        }
        None
    }

    /// Read DIB data and convert to PNG
    fn read_dib_format(format: u32) -> Option<ImageData> {
        unsafe {
            if OpenClipboard(HWND::default()).is_err() {
                return None;
            }

            let result = (|| {
                let handle: HANDLE = match GetClipboardData(format) {
                    Ok(h) => h,
                    Err(_) => return None,
                };

                if handle.0.is_null() {
                    return None;
                }

                let hglobal = HGLOBAL(handle.0);
                let size = GlobalSize(hglobal);
                if size == 0 {
                    return None;
                }

                let ptr = GlobalLock(hglobal);
                if ptr.is_null() {
                    return None;
                }

                let data = std::slice::from_raw_parts(ptr as *const u8, size);
                let dib_data = data.to_vec();
                let _ = GlobalUnlock(hglobal);

                if dib_data.len() < 12 {
                    return None;
                }

                let width = u32::from_le_bytes([dib_data[4], dib_data[5], dib_data[6], dib_data[7]]);
                let height_raw = i32::from_le_bytes([dib_data[8], dib_data[9], dib_data[10], dib_data[11]]);
                let height = height_raw.unsigned_abs();

                // Convert DIB to PNG
                let png_data = dib_to_png(&dib_data)?;

                Some(ImageData {
                    png_data,
                    width,
                    height,
                })
            })();

            let _ = CloseClipboard();
            result
        }
    }

    /// Convert DIB data to PNG format
    fn dib_to_png(dib_data: &[u8]) -> Option<Vec<u8>> {
        // Create BMP from DIB
        let file_header_size = 14;
        let file_size = file_header_size + dib_data.len();
        let mut bmp = Vec::with_capacity(file_size);

        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
        bmp.extend_from_slice(&[0u8; 4]);

        let dib_header_size = if dib_data.len() >= 4 {
            u32::from_le_bytes([dib_data[0], dib_data[1], dib_data[2], dib_data[3]])
        } else {
            40
        };
        let pixel_offset = file_header_size as u32 + dib_header_size;
        bmp.extend_from_slice(&pixel_offset.to_le_bytes());
        bmp.extend_from_slice(dib_data);

        // Decode BMP and encode as PNG
        let img = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp).ok()?;
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).ok()?;
        Some(png_bytes)
    }

    /// Read file list from clipboard
    pub fn read_files() -> Option<Vec<String>> {
        match get_clipboard::<Vec<String>, _>(formats::FileList) {
            Ok(files) if !files.is_empty() => Some(files),
            _ => None,
        }
    }

    /// Read clipboard content based on detected format
    pub fn read_clipboard() -> ClipboardContent {
        let content_type = detect_format();

        match content_type {
            ContentType::Text => {
                if let Some(text) = read_text() {
                    ClipboardContent::Text(text)
                } else {
                    ClipboardContent::Empty
                }
            }
            ContentType::Image => {
                if let Some(image) = read_image() {
                    ClipboardContent::Image(image)
                } else {
                    ClipboardContent::Empty
                }
            }
            ContentType::Files => {
                if let Some(files) = read_files() {
                    ClipboardContent::Files(files)
                } else {
                    ClipboardContent::Empty
                }
            }
        }
    }

    /// Get clipboard text (simple API)
    pub fn get_clipboard_text() -> Result<String, String> {
        read_text().ok_or_else(|| "No text in clipboard".to_string())
    }

    /// Set clipboard text
    pub fn set_clipboard_text(text: &str) -> Result<(), String> {
        clipboard_win::set_clipboard(formats::Unicode, text)
            .map_err(|e| format!("Failed to set clipboard: {}", e))
    }
}

// ============================================================================
// macOS Implementation (using arboard + native NSPasteboard for files)
// ============================================================================
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use arboard::Clipboard;
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSString, NSURL};

    /// Get the pasteboard change count (increments on every clipboard change)
    /// This is the most reliable way to detect clipboard changes on macOS
    pub fn get_change_count() -> isize {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            pasteboard.changeCount()
        }
    }

    /// Check if the pasteboard has any content (types > 0)
    /// Used to filter out clipboard clears and transient states
    pub fn pasteboard_has_content() -> bool {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            if let Some(types) = pasteboard.types() {
                types.count() > 0
            } else {
                false
            }
        }
    }

    /// Log all available UTI types on the pasteboard (for debugging)
    /// This helps identify what formats are available that we might be missing
    fn log_pasteboard_types() {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let change_count = pasteboard.changeCount();
            eprintln!("│   [PASTEBOARD changeCount: {}]", change_count);

            let types = pasteboard.types();

            eprintln!("│   [PASTEBOARD TYPES AVAILABLE]:");
            if let Some(types) = types {
                let count = types.count();
                eprintln!("│     (count: {})", count);
                for i in 0..count {
                    let t: &NSString = &types.objectAtIndex(i);
                    eprintln!("│     - {}", t.to_string());
                }
            } else {
                eprintln!("│     (none - types() returned None)");
            }

            // Also check specific common types directly
            eprintln!("│   [DIRECT TYPE CHECKS]:");
            let check_types = [
                "public.utf8-plain-text",
                "public.tiff",
                "public.png",
                "public.file-url",
                "com.apple.pboard.promised-file-url",
                "dyn.ah62d4rv4gu8y63n2nuuhg5pbsm4ca6dbsr4gnkduqf31k3pcr7u1e3basv61a3k",
            ];
            for type_str in check_types {
                let ns_type = NSString::from_str(type_str);
                let data = pasteboard.dataForType(&ns_type);
                let has_data = data.is_some();
                let data_len = data.map(|d| d.len()).unwrap_or(0);
                eprintln!("│     {} : {} ({} bytes)", type_str, if has_data { "YES" } else { "NO" }, data_len);
            }
        }
    }

    /// Check if files are available on the pasteboard
    fn has_files_on_pasteboard() -> bool {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let file_url_type = NSString::from_str("public.file-url");
            let types = pasteboard.types();

            if let Some(types) = types {
                for i in 0..types.count() {
                    let t = types.objectAtIndex(i);
                    if t.isEqualToString(&file_url_type) {
                        return true;
                    }
                }
            }
            false
        }
    }

    /// Detect the primary content type available on the clipboard
    pub fn detect_format() -> ContentType {
        // Check for files first (like Windows implementation)
        if has_files_on_pasteboard() {
            return ContentType::Files;
        }

        // Try to get a clipboard instance for other types
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return ContentType::Text,
        };

        // Check for image
        if clipboard.get_image().is_ok() {
            return ContentType::Image;
        }

        // Check for text
        if clipboard.get_text().is_ok() {
            return ContentType::Text;
        }

        // Default to text
        ContentType::Text
    }

    /// Read text from clipboard
    pub fn read_text() -> Option<String> {
        eprintln!("[DEBUG read_text] Attempting to read text...");
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[DEBUG read_text]   Failed to create clipboard: {:?}", e);
                return None;
            }
        };
        match clipboard.get_text() {
            Ok(text) if !text.is_empty() => {
                eprintln!("[DEBUG read_text]   Got text: {} chars", text.len());
                Some(text)
            }
            Ok(_) => {
                eprintln!("[DEBUG read_text]   Got empty text");
                None
            }
            Err(e) => {
                eprintln!("[DEBUG read_text]   Error: {:?}", e);
                None
            }
        }
    }

    /// Read image data from clipboard
    /// Tries multiple methods to capture images:
    /// 1. arboard (cross-platform, handles most cases)
    /// 2. Native NSPasteboard TIFF data (for apps that only provide TIFF)
    /// 3. Native NSPasteboard PNG data (for PNG-specific sources)
    pub fn read_image() -> Option<ImageData> {
        eprintln!("[DEBUG read_image] Attempting to read image from clipboard...");

        // Method 1: Try arboard first (handles most cases)
        if let Some(img_data) = read_image_arboard() {
            return Some(img_data);
        }

        // Method 2: Try native NSPasteboard for TIFF data
        if let Some(img_data) = read_image_native_tiff() {
            return Some(img_data);
        }

        // Method 3: Try native NSPasteboard for PNG data
        if let Some(img_data) = read_image_native_png() {
            return Some(img_data);
        }

        eprintln!("[DEBUG read_image]   All methods failed - no image captured");
        None
    }

    /// Try reading image via arboard
    fn read_image_arboard() -> Option<ImageData> {
        let mut clipboard = Clipboard::new().ok()?;
        let img_data = match clipboard.get_image() {
            Ok(data) => {
                eprintln!("[DEBUG read_image]   arboard: Got image {}x{}, {} bytes RGBA", data.width, data.height, data.bytes.len());
                data
            }
            Err(e) => {
                eprintln!("[DEBUG read_image]   arboard: No image - {:?}", e);
                return None;
            }
        };

        // Convert RGBA to PNG
        let width = img_data.width as u32;
        let height = img_data.height as u32;

        // Create image from raw RGBA bytes
        let img = image::RgbaImage::from_raw(width, height, img_data.bytes.into_owned())?;
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        // Encode as PNG
        let mut png_data = Vec::new();
        dynamic_img
            .write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
            .ok()?;

        eprintln!("[DEBUG read_image]   arboard: Encoded to PNG: {} bytes", png_data.len());

        Some(ImageData {
            png_data,
            width,
            height,
        })
    }

    /// Try reading image via native NSPasteboard TIFF data
    /// Some apps (like Preview, some browsers) provide TIFF format
    fn read_image_native_tiff() -> Option<ImageData> {
        use objc2_app_kit::NSBitmapImageRep;

        eprintln!("[DEBUG read_image]   Trying NSPasteboard TIFF...");

        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let tiff_type = NSString::from_str("public.tiff");

            // Check if TIFF data is available
            let data = match pasteboard.dataForType(&tiff_type) {
                Some(d) => d,
                None => {
                    eprintln!("[DEBUG read_image]   NSPasteboard TIFF: No data available");
                    return None;
                }
            };

            eprintln!("[DEBUG read_image]   NSPasteboard TIFF: Found {} bytes", data.len());

            // Create bitmap rep from TIFF data
            let bitmap_rep = match NSBitmapImageRep::imageRepWithData(&data) {
                Some(rep) => rep,
                None => {
                    eprintln!("[DEBUG read_image]   NSPasteboard TIFF: Failed to create bitmap rep");
                    return None;
                }
            };

            let width = bitmap_rep.pixelsWide() as u32;
            let height = bitmap_rep.pixelsHigh() as u32;

            eprintln!("[DEBUG read_image]   NSPasteboard TIFF: Decoded {}x{}", width, height);

            // Convert to PNG
            use objc2_app_kit::NSBitmapImageFileType;
            use objc2_foundation::NSDictionary;

            let png_data = match bitmap_rep.representationUsingType_properties(
                NSBitmapImageFileType::PNG,
                &NSDictionary::new(),
            ) {
                Some(d) => d,
                None => {
                    eprintln!("[DEBUG read_image]   NSPasteboard TIFF: Failed to convert to PNG");
                    return None;
                }
            };

            let png_bytes = png_data.bytes().to_vec();
            eprintln!("[DEBUG read_image]   NSPasteboard TIFF: Encoded to PNG: {} bytes", png_bytes.len());

            Some(ImageData {
                png_data: png_bytes,
                width,
                height,
            })
        }
    }

    /// Try reading image via native NSPasteboard PNG data
    fn read_image_native_png() -> Option<ImageData> {
        eprintln!("[DEBUG read_image]   Trying NSPasteboard PNG...");

        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let png_type = NSString::from_str("public.png");

            // Check if PNG data is available
            let data = match pasteboard.dataForType(&png_type) {
                Some(d) => d,
                None => {
                    eprintln!("[DEBUG read_image]   NSPasteboard PNG: No data available");
                    return None;
                }
            };
            let png_bytes = data.bytes().to_vec();

            eprintln!("[DEBUG read_image]   NSPasteboard PNG: Found {} bytes", png_bytes.len());

            // Decode to get dimensions
            let img = match image::load_from_memory(&png_bytes) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("[DEBUG read_image]   NSPasteboard PNG: Failed to decode: {}", e);
                    return None;
                }
            };
            let width = img.width();
            let height = img.height();

            eprintln!("[DEBUG read_image]   NSPasteboard PNG: Decoded {}x{}", width, height);

            Some(ImageData {
                png_data: png_bytes,
                width,
                height,
            })
        }
    }

    /// Read file list from clipboard using native NSPasteboard
    pub fn read_files() -> Option<Vec<String>> {
        use percent_encoding::percent_decode_str;

        eprintln!("[DEBUG read_files] Attempting to read files...");

        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let file_url_type = NSString::from_str("public.file-url");

            // Get pasteboard items
            let items = match pasteboard.pasteboardItems() {
                Some(i) => i,
                None => {
                    eprintln!("[DEBUG read_files]   No pasteboard items");
                    return None;
                }
            };

            let item_count = items.count();
            eprintln!("[DEBUG read_files]   Found {} pasteboard items", item_count);

            let mut file_paths: Vec<String> = Vec::new();

            for i in 0..item_count {
                let item = items.objectAtIndex(i);

                // Log all types available for this item
                let types = item.types();
                let type_count = types.count();
                eprintln!("[DEBUG read_files]   Item {} has {} types", i, type_count);
                for j in 0..type_count {
                    let t: &NSString = &types.objectAtIndex(j);
                    eprintln!("[DEBUG read_files]     - {}", t.to_string());
                }

                // Get the file URL string from the item
                if let Some(url_string) = item.stringForType(&file_url_type) {
                    // Convert file:// URL to path
                    let url_str: String = url_string.to_string();
                    eprintln!("[DEBUG read_files]   Item {} file URL: {}", i, url_str);
                    if let Some(nsurl) = NSURL::URLWithString(&NSString::from_str(&url_str)) {
                        if let Some(path) = nsurl.path() {
                            let path_str: String = path.to_string();
                            if !path_str.is_empty() {
                                // Decode URL-encoded characters (e.g., %20 -> space)
                                let decoded_path = percent_decode_str(&path_str)
                                    .decode_utf8()
                                    .ok()
                                    .map(|s| s.to_string())
                                    .unwrap_or(path_str);
                                eprintln!("[DEBUG read_files]   Decoded path: {}", decoded_path);
                                file_paths.push(decoded_path);
                            }
                        }
                    }
                } else {
                    eprintln!("[DEBUG read_files]   Item {} has no public.file-url", i);
                }
            }

            if file_paths.is_empty() {
                eprintln!("[DEBUG read_files]   No file paths found");
                None
            } else {
                eprintln!("[DEBUG read_files]   Found {} files: {:?}", file_paths.len(), file_paths);
                Some(file_paths)
            }
        }
    }

    /// Read clipboard content based on detected format
    /// Priority order depends on content:
    /// - If files exist on disk → treat as FILES (preserves original filename)
    /// - Otherwise → Image -> Text
    pub fn read_clipboard() -> ClipboardContent {
        eprintln!("┌─────────────────────────────────────────────────────────────");
        eprintln!("│ [DEBUG read_clipboard] Checking clipboard content...");

        // LOG ALL AVAILABLE TYPES for debugging capture failures
        log_pasteboard_types();

        // Check what's available
        let has_files = read_files();
        let has_image = read_image();

        eprintln!("│   has_files: {:?}", has_files.as_ref().map(|f| f.clone()));
        eprintln!("│   has_image: {}", has_image.is_some());

        // If we have FILES that exist on disk, prioritize them (preserves original filename)
        // This handles the case of copying a file from Finder
        if let Some(ref file_list) = has_files {
            // Check if files actually exist on disk
            let files_exist = file_list.iter().all(|f| std::path::Path::new(f).exists());
            eprintln!("│   files_exist on disk: {}", files_exist);

            if files_exist && !file_list.is_empty() {
                eprintln!("│ → Using FILES (original paths preserved): {:?}", file_list);
                eprintln!("└─────────────────────────────────────────────────────────────");
                return ClipboardContent::Files(file_list.clone());
            } else if !files_exist {
                eprintln!("│   WARNING: Files detected but don't exist on disk!");
                for f in file_list {
                    eprintln!("│     - {} (exists: {})", f, std::path::Path::new(f).exists());
                }
            }
        }

        // Otherwise, check for image data (screenshots, copied images from apps)
        if let Some(img) = has_image {
            eprintln!("│ → Found IMAGE: {}x{}, {} bytes PNG", img.width, img.height, img.png_data.len());
            eprintln!("└─────────────────────────────────────────────────────────────");
            return ClipboardContent::Image(img);
        }

        // Fallback to files even if they don't exist (edge case)
        if let Some(file_list) = has_files {
            eprintln!("│ → Found FILES (fallback): {:?}", file_list);
            eprintln!("└─────────────────────────────────────────────────────────────");
            return ClipboardContent::Files(file_list);
        }

        // Check text last
        if let Some(text) = read_text() {
            let preview = if text.len() > 50 { &text[..50] } else { &text };
            eprintln!("│ → Found TEXT: {}... ({} chars)", preview, text.len());
            eprintln!("└─────────────────────────────────────────────────────────────");
            return ClipboardContent::Text(text);
        }

        eprintln!("│ → EMPTY clipboard (no files, no image, no text)");
        eprintln!("│   This may indicate an unsupported UTI type - check types above");
        eprintln!("└─────────────────────────────────────────────────────────────");
        ClipboardContent::Empty
    }

    /// Get clipboard text (simple API)
    pub fn get_clipboard_text() -> Result<String, String> {
        read_text().ok_or_else(|| "No text in clipboard".to_string())
    }

    /// Set clipboard text
    pub fn set_clipboard_text(text: &str) -> Result<(), String> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?;
        clipboard
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard: {}", e))
    }
}

// ============================================================================
// Public API (re-exports platform implementation)
// ============================================================================

pub use platform::*;
