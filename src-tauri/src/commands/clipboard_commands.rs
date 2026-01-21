use crate::clipboard::clipboard_reader;
use crate::models::ClipboardItem;
use crate::AppState;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;
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
        crate::models::ContentType::Files | crate::models::ContentType::Audio => {
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
