// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod windows_api;
mod commands;

// use windows_connection::clipboard::get_clipboard_text;
use commands::clipboard_commands::get_clipboard;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

