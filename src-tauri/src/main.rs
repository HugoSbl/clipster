// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod storage;
mod windows_api;

use commands::clipboard_commands::{
    assign_to_pinboard, clear_clipboard_history, copy_to_clipboard, delete_clipboard_item,
    get_clipboard, get_clipboard_count, get_clipboard_history, get_clipboard_item,
    get_image_data, search_clipboard, toggle_favorite,
};
use commands::pinboard_commands::{
    add_item_to_pinboard, create_pinboard, delete_pinboard, get_pinboard, get_pinboard_items,
    get_pinboards, remove_item_from_pinboard, reorder_pinboards, update_pinboard,
};
use commands::settings_commands::{
    get_history_limit, get_settings, set_history_limit, update_setting,
};
use std::sync::Arc;
use storage::Database;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use windows_api::clipboard_monitor;

/// Application state holding the database connection
pub struct AppState {
    pub db: Arc<Database>,
}

/// Toggle window visibility - show if hidden, hide if visible
fn toggle_window_visibility(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            // Window is visible, hide it
            let _ = window.hide();
            println!("Window hidden");
        } else {
            // Window is hidden, show and focus it
            let _ = window.show();
            let _ = window.set_focus();
            println!("Window shown and focused");
        }
    }
}

fn main() {
    // Initialize database
    let db = Database::new().expect("Failed to initialize database");
    let db = Arc::new(db);

    // Define the global shortcut
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut_pressed, _event| {
                    if shortcut_pressed == &shortcut {
                        toggle_window_visibility(app);
                    }
                })
                .build(),
        )
        .manage(AppState { db: db.clone() })
        .setup(move |app| {
            // Start clipboard monitoring
            let app_handle = app.handle().clone();
            if let Err(e) = clipboard_monitor::start_monitoring(app_handle.clone(), db.clone()) {
                eprintln!("Failed to start clipboard monitor: {}", e);
            } else {
                println!("Clipboard monitor started successfully");
            }

            // Register global shortcut (Ctrl+Shift+V)
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
            app.global_shortcut().register(shortcut)?;
            println!("Global shortcut Ctrl+Shift+V registered");

            // Create system tray menu
            let show_hide = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_hide, &settings, &quit])?;

            // Create tray icon
            let app_handle_for_tray = app.handle().clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Clipster - Clipboard Manager")
                .on_tray_icon_event(move |_tray, event| {
                    // Left click toggles window visibility
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window_visibility(&app_handle_for_tray);
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_hide" => {
                        toggle_window_visibility(app);
                    }
                    "settings" => {
                        println!("Settings clicked");
                        // Show window and emit event to open settings modal
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("open-settings", ());
                        }
                    }
                    "quit" => {
                        println!("Quit requested");
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            println!("System tray created");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Clipboard commands
            get_clipboard,
            get_clipboard_history,
            get_clipboard_item,
            delete_clipboard_item,
            search_clipboard,
            clear_clipboard_history,
            copy_to_clipboard,
            toggle_favorite,
            assign_to_pinboard,
            get_clipboard_count,
            get_image_data,
            // Pinboard commands
            get_pinboards,
            get_pinboard,
            create_pinboard,
            update_pinboard,
            delete_pinboard,
            reorder_pinboards,
            get_pinboard_items,
            add_item_to_pinboard,
            remove_item_from_pinboard,
            // Settings commands
            get_settings,
            update_setting,
            get_history_limit,
            set_history_limit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

