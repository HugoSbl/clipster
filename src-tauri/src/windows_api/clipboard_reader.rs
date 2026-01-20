//! Multi-format clipboard reader for Windows
//!
//! Supports reading:
//! - CF_UNICODETEXT (13): Unicode text
//! - CF_DIB (8): Device Independent Bitmap (images)
//! - CF_HDROP (15): File list (copied files)

use crate::models::ContentType;
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
    /// Raw DIB data (without BMP file header)
    pub dib_data: Vec<u8>,
    /// Width in pixels (parsed from BITMAPINFOHEADER)
    pub width: u32,
    /// Height in pixels (parsed from BITMAPINFOHEADER)
    pub height: u32,
}

impl ImageData {
    /// Convert DIB data to a complete BMP file with header
    pub fn to_bmp(&self) -> Vec<u8> {
        let file_header_size = 14;
        let file_size = file_header_size + self.dib_data.len();

        let mut bmp = Vec::with_capacity(file_size);

        // BMP file header (14 bytes)
        bmp.extend_from_slice(b"BM"); // Signature
        bmp.extend_from_slice(&(file_size as u32).to_le_bytes()); // File size
        bmp.extend_from_slice(&[0u8; 4]); // Reserved

        // Pixel data offset - file header (14) + DIB header size (first 4 bytes of dib_data)
        let dib_header_size = if self.dib_data.len() >= 4 {
            u32::from_le_bytes([self.dib_data[0], self.dib_data[1], self.dib_data[2], self.dib_data[3]])
        } else {
            40 // Default BITMAPINFOHEADER size
        };
        let pixel_offset = file_header_size as u32 + dib_header_size;
        bmp.extend_from_slice(&pixel_offset.to_le_bytes());

        // Append DIB data
        bmp.extend_from_slice(&self.dib_data);

        bmp
    }
}

/// Detect the primary content type available on the clipboard
/// Priority: Text > Image > Files
pub fn detect_format() -> ContentType {
    // Check for text first (most common)
    if is_format_avail(formats::CF_UNICODETEXT) {
        return ContentType::Text;
    }

    // Check for image (DIB format)
    if is_raw_avail(clipboard_formats::CF_DIB) || is_raw_avail(clipboard_formats::CF_DIBV5) {
        return ContentType::Image;
    }

    // Check for files
    if is_format_avail(formats::CF_HDROP) {
        return ContentType::Files;
    }

    // Default to text (will be empty if nothing available)
    ContentType::Text
}

/// Check which formats are currently available on the clipboard
pub fn available_formats() -> Vec<&'static str> {
    let mut formats_list = Vec::new();

    if is_format_avail(formats::CF_UNICODETEXT) {
        formats_list.push("CF_UNICODETEXT");
    }
    if is_raw_avail(clipboard_formats::CF_DIB) {
        formats_list.push("CF_DIB");
    }
    if is_raw_avail(clipboard_formats::CF_DIBV5) {
        formats_list.push("CF_DIBV5");
    }
    if is_format_avail(formats::CF_HDROP) {
        formats_list.push("CF_HDROP");
    }
    if is_raw_avail(clipboard_formats::CF_BITMAP) {
        formats_list.push("CF_BITMAP");
    }

    formats_list
}

/// Read text from clipboard (CF_UNICODETEXT)
pub fn read_text() -> Option<String> {
    match get_clipboard::<String, _>(formats::Unicode) {
        Ok(text) if !text.is_empty() => Some(text),
        _ => None,
    }
}

/// Read image data from clipboard (CF_DIB)
/// Returns raw DIB data that needs to be converted to a proper image format
pub fn read_image() -> Option<ImageData> {
    // Try CF_DIB first (most common for screenshots)
    if let Some(data) = read_dib_format(clipboard_formats::CF_DIB) {
        return Some(data);
    }

    // Fall back to CF_DIBV5 if available
    if let Some(data) = read_dib_format(clipboard_formats::CF_DIBV5) {
        return Some(data);
    }

    None
}

/// Read DIB data from a specific clipboard format using Windows API
fn read_dib_format(format: u32) -> Option<ImageData> {
    unsafe {
        // Open clipboard
        if OpenClipboard(HWND::default()).is_err() {
            return None;
        }

        let result = (|| {
            // Get clipboard data handle
            let handle: HANDLE = match GetClipboardData(format) {
                Ok(h) => h,
                Err(_) => return None,
            };

            if handle.0.is_null() {
                return None;
            }

            // Convert HANDLE to HGLOBAL (they're compatible for clipboard data)
            let hglobal = HGLOBAL(handle.0);

            // Get data size
            let size = GlobalSize(hglobal);
            if size == 0 {
                return None;
            }

            // Lock memory and copy data
            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                return None;
            }

            let data = std::slice::from_raw_parts(ptr as *const u8, size);
            let dib_data = data.to_vec();

            let _ = GlobalUnlock(hglobal);

            // Parse BITMAPINFOHEADER to get dimensions
            if dib_data.len() < 12 {
                return None;
            }

            let width = u32::from_le_bytes([dib_data[4], dib_data[5], dib_data[6], dib_data[7]]);
            let height_raw = i32::from_le_bytes([dib_data[8], dib_data[9], dib_data[10], dib_data[11]]);
            let height = height_raw.unsigned_abs();

            Some(ImageData {
                dib_data,
                width,
                height,
            })
        })();

        // Always close clipboard
        let _ = CloseClipboard();

        result
    }
}

/// Read file list from clipboard (CF_HDROP)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_returns_valid_type() {
        // Just ensure it doesn't panic
        let _ = detect_format();
    }

    #[test]
    fn test_available_formats_returns_list() {
        // Just ensure it doesn't panic
        let formats = available_formats();
        println!("Available formats: {:?}", formats);
    }

    #[test]
    fn test_read_clipboard_returns_content() {
        // Just ensure it doesn't panic
        let content = read_clipboard();
        match content {
            ClipboardContent::Text(t) => println!("Text: {} chars", t.len()),
            ClipboardContent::Image(i) => println!("Image: {}x{}", i.width, i.height),
            ClipboardContent::Files(f) => println!("Files: {:?}", f),
            ClipboardContent::Empty => println!("Empty"),
        }
    }

    #[test]
    fn test_image_to_bmp() {
        // Create a minimal fake BITMAPINFOHEADER
        let mut fake_dib = vec![0u8; 40]; // Minimum header size
        fake_dib[0..4].copy_from_slice(&40u32.to_le_bytes()); // biSize
        fake_dib[4..8].copy_from_slice(&100u32.to_le_bytes()); // biWidth
        fake_dib[8..12].copy_from_slice(&50i32.to_le_bytes()); // biHeight

        let image_data = ImageData {
            dib_data: fake_dib,
            width: 100,
            height: 50,
        };

        let bmp = image_data.to_bmp();

        // Check BMP signature
        assert_eq!(&bmp[0..2], b"BM");

        // Check file size
        let file_size = u32::from_le_bytes([bmp[2], bmp[3], bmp[4], bmp[5]]);
        assert_eq!(file_size as usize, bmp.len());
    }
}
