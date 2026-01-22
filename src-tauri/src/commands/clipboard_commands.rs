use crate::clipboard::clipboard_reader;
use crate::models::ClipboardItem;
use crate::AppState;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;
use std::path::Path;
use tauri::State;

/// Get current clipboard text (legacy command)
#[tauri::command]
pub fn get_clipboard() -> Result<String, String> {
    clipboard_reader::get_clipboard_text()
}

/// Get clipboard history with pagination
#[tauri::command]
pub fn get_clipboard_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    state.db.get_items(limit, offset)
}

/// Get a single clipboard item by ID
#[tauri::command]
pub fn get_clipboard_item(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<ClipboardItem>, String> {
    state.db.get_item(&id)
}

/// Delete a clipboard item by ID
#[tauri::command]
pub fn delete_clipboard_item(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    state.db.delete_item(&id)
}

/// Search clipboard history by text content
#[tauri::command]
pub fn search_clipboard(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = limit.unwrap_or(50);
    state.db.search_items(&query, limit)
}

/// Clear all clipboard history (except favorites and pinned items)
#[tauri::command]
pub fn clear_clipboard_history(state: State<'_, AppState>) -> Result<usize, String> {
    state.db.clear_history()
}

/// Copy an item back to the system clipboard
#[tauri::command]
pub fn copy_to_clipboard(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let item = state
        .db
        .get_item(&id)?
        .ok_or_else(|| "Item not found".to_string())?;

    match item.content_type {
        crate::models::ContentType::Text | crate::models::ContentType::Link => {
            if let Some(text) = &item.content_text {
                clipboard_reader::set_clipboard_text(text)?;
            } else {
                return Err("No text content in item".to_string());
            }
        }
        crate::models::ContentType::Image => {
            // TODO: Implement image copy
            return Err("Image copy not yet implemented".to_string());
        }
        crate::models::ContentType::Files
        | crate::models::ContentType::Audio
        | crate::models::ContentType::Documents => {
            // TODO: Implement file copy
            return Err("File copy not yet implemented".to_string());
        }
    }

    Ok(())
}

/// Toggle favorite status of an item
#[tauri::command]
pub fn toggle_favorite(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    state.db.toggle_item_favorite(&id)
}

/// Assign an item to a pinboard
#[tauri::command]
pub fn assign_to_pinboard(
    state: State<'_, AppState>,
    item_id: String,
    pinboard_id: Option<String>,
) -> Result<bool, String> {
    state.db.update_item_pinboard(&item_id, pinboard_id.as_deref())
}

/// Get total count of clipboard items
#[tauri::command]
pub fn get_clipboard_count(state: State<'_, AppState>) -> Result<usize, String> {
    state.db.count_items()
}

/// Get full image data as base64 encoded PNG
#[tauri::command]
pub fn get_image_data(
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let item = state
        .db
        .get_item(&id)?
        .ok_or_else(|| "Item not found".to_string())?;

    // Verify it's an image item
    if item.content_type != crate::models::ContentType::Image {
        return Err("Item is not an image".to_string());
    }

    // Get the image path
    let image_path = item
        .image_path
        .ok_or_else(|| "Image path not found".to_string())?;

    // Read the image file
    let image_bytes = fs::read(&image_path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;

    // Encode as base64
    Ok(BASE64.encode(&image_bytes))
}

/// Prepare an image file for drag by copying it to temp with a readable filename
/// Returns (image_path, icon_path) - both paths for the drag operation
#[tauri::command]
pub fn prepare_image_for_drag(
    source_path: String,
    readable_filename: String,
) -> Result<(String, String), String> {
    eprintln!("═══════════════════════════════════════════════════════════");
    eprintln!("[DEBUG prepare_image_for_drag] CALLED");
    eprintln!("[DEBUG]   source_path: {}", source_path);
    eprintln!("[DEBUG]   readable_filename: {}", readable_filename);

    // Verify source file exists
    let source = Path::new(&source_path);
    if !source.exists() {
        eprintln!("[DEBUG]   ERROR: Source file not found!");
        return Err(format!("Source file not found: {}", source_path));
    }

    // Get source file size
    let source_size = fs::metadata(source).map(|m| m.len()).unwrap_or(0);
    eprintln!("[DEBUG]   source file exists, size: {} bytes", source_size);

    // Get system temp directory
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(&readable_filename);
    eprintln!("[DEBUG]   temp_path: {:?}", temp_path);

    // Copy file to temp location with readable name
    eprintln!("[DEBUG]   Copying file...");
    let bytes_copied = fs::copy(source, &temp_path)
        .map_err(|e| format!("Failed to copy file to temp: {}", e))?;
    eprintln!("[DEBUG]   Copied {} bytes to temp", bytes_copied);

    // Verify the copy
    let temp_size = fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
    eprintln!("[DEBUG]   Verification - temp file size: {} bytes", temp_size);

    if temp_size != source_size {
        eprintln!("[DEBUG]   WARNING: Size mismatch! source={} temp={}", source_size, temp_size);
    }

    // On macOS, remove quarantine attribute so Quick Look and Finder work correctly
    #[cfg(target_os = "macos")]
    {
        eprintln!("[DEBUG]   Removing quarantine xattr...");
        let _ = std::process::Command::new("xattr")
            .args(["-d", "com.apple.quarantine", temp_path.to_str().unwrap_or("")])
            .output();
    }

    let temp_path_str = temp_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert path to string".to_string())?;

    // Create a small thumbnail for drag icon (separate from actual file)
    let icon_filename = format!("icon_{}", readable_filename);
    let icon_path = temp_dir.join(&icon_filename);
    eprintln!("[DEBUG]   Creating drag icon at: {:?}", icon_path);

    // Create 64x64 thumbnail for drag preview
    if let Ok(img) = image::open(source) {
        let thumbnail = img.thumbnail(64, 64);
        let _ = thumbnail.save_with_format(&icon_path, image::ImageFormat::Png);
        eprintln!("[DEBUG]   Created 64x64 thumbnail icon");
    } else {
        // If thumbnail fails, copy the original (fallback)
        let _ = fs::copy(source, &icon_path);
        eprintln!("[DEBUG]   Thumbnail failed, copied original as icon");
    }

    let icon_path_str = icon_path
        .to_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| temp_path_str.clone());

    eprintln!("[DEBUG]   RETURNING:");
    eprintln!("[DEBUG]     image_path (item): {}", temp_path_str);
    eprintln!("[DEBUG]     icon_path: {}", icon_path_str);
    eprintln!("═══════════════════════════════════════════════════════════");

    Ok((temp_path_str, icon_path_str))
}
