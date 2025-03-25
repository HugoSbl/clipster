
use crate::windows_api::windows_api;

#[tauri::command]
pub fn get_clipboard() -> Result<String, String> {
    match windows_api::get_clipboard_datas() {
        Ok(text) => Ok(text),
        Err(e) => Err(e),
    }
}
