use crate::models::{ClipboardItem, Pinboard};
use crate::AppState;
use tauri::State;

/// Get all pinboards ordered by position
#[tauri::command]
pub fn get_pinboards(state: State<'_, AppState>) -> Result<Vec<Pinboard>, String> {
    state.db.get_pinboards()
}

/// Get a single pinboard by ID
#[tauri::command]
pub fn get_pinboard(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<Pinboard>, String> {
    state.db.get_pinboard(&id)
}

/// Create a new pinboard
#[tauri::command]
pub fn create_pinboard(
    state: State<'_, AppState>,
    name: String,
    icon: Option<String>,
) -> Result<Pinboard, String> {
    // Get current pinboards to determine position
    let pinboards = state.db.get_pinboards()?;
    let position = pinboards.len() as i32;

    let pinboard = Pinboard::new(name, icon, position);
    state.db.insert_pinboard(&pinboard)?;

    Ok(pinboard)
}

/// Update an existing pinboard
#[tauri::command]
pub fn update_pinboard(
    state: State<'_, AppState>,
    id: String,
    name: String,
    icon: Option<String>,
) -> Result<bool, String> {
    state.db.update_pinboard(&id, &name, icon.as_deref())
}

/// Delete a pinboard
#[tauri::command]
pub fn delete_pinboard(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    state.db.delete_pinboard(&id)
}

/// Reorder pinboards by providing list of IDs in desired order
#[tauri::command]
pub fn reorder_pinboards(
    state: State<'_, AppState>,
    pinboard_ids: Vec<String>,
) -> Result<(), String> {
    state.db.reorder_pinboards(&pinboard_ids)
}

/// Get items in a specific pinboard
#[tauri::command]
pub fn get_pinboard_items(
    state: State<'_, AppState>,
    pinboard_id: String,
    limit: Option<usize>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = limit.unwrap_or(100);
    state.db.get_pinboard_items(&pinboard_id, limit)
}

/// Add an item to a pinboard
#[tauri::command]
pub fn add_item_to_pinboard(
    state: State<'_, AppState>,
    item_id: String,
    pinboard_id: String,
) -> Result<bool, String> {
    state.db.update_item_pinboard(&item_id, Some(&pinboard_id))
}

/// Remove an item from its pinboard (set pinboard_id to NULL)
#[tauri::command]
pub fn remove_item_from_pinboard(
    state: State<'_, AppState>,
    item_id: String,
) -> Result<bool, String> {
    state.db.update_item_pinboard(&item_id, None)
}
