use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Settings structure returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub shortcut: String,
    pub history_limit: u32,
    pub start_hidden: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shortcut: "Ctrl+Shift+V".to_string(),
            history_limit: 500,
            start_hidden: false,
        }
    }
}

/// Get all settings
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let shortcut = state
        .db
        .get_setting("shortcut")?
        .unwrap_or_else(|| "Ctrl+Shift+V".to_string());

    let history_limit_str = state
        .db
        .get_setting("history_limit")?
        .unwrap_or_else(|| "500".to_string());
    let history_limit = history_limit_str.parse().unwrap_or(500);

    let start_hidden_str = state
        .db
        .get_setting("start_hidden")?
        .unwrap_or_else(|| "false".to_string());
    let start_hidden = start_hidden_str == "true";

    Ok(AppSettings {
        shortcut,
        history_limit,
        start_hidden,
    })
}

/// Update a single setting
#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state.db.set_setting(&key, &value)
}

/// Get history limit setting
#[tauri::command]
pub fn get_history_limit(state: State<'_, AppState>) -> Result<u32, String> {
    state.db.get_history_limit().map(|v| v as u32)
}

/// Set history limit and prune if necessary
#[tauri::command]
pub fn set_history_limit(state: State<'_, AppState>, limit: u32) -> Result<(), String> {
    state.db.set_setting("history_limit", &limit.to_string())?;
    // Prune old items if over limit
    state.db.prune_oldest(limit as usize)?;
    Ok(())
}
