// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod models;
mod storage;

use commands::clipboard_commands::{
    assign_to_pinboard, clear_clipboard_history, copy_to_clipboard, create_drag_icon,
    create_temp_link_file, create_temp_text_file, delete_clipboard_item, get_clipboard,
    get_clipboard_count, get_clipboard_history, get_clipboard_item, get_image_data,
    prepare_image_for_drag, search_clipboard, toggle_favorite,
};
use commands::pinboard_commands::{
    add_item_to_pinboard, create_pinboard, delete_pinboard, get_pinboard, get_pinboard_items,
    get_pinboards, remove_item_from_pinboard, reorder_pinboards, update_pinboard,
};
use commands::settings_commands::{
    get_history_limit, get_settings, set_history_limit, update_setting,
};
use commands::window_commands::{
    ensure_overlay, hide_window, quit_app, reposition_to_cursor_monitor, show_window,
};
use std::sync::Arc;
use storage::Database;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use clipboard::clipboard_monitor;
use tauri_plugin_autostart::MacosLauncher;

/// Application state holding the database connection
pub struct AppState {
    pub db: Arc<Database>,
}

/// Toggle window visibility - show if hidden, hide if visible
fn toggle_window_visibility(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            println!("Window hidden");
        } else {
            reposition_to_cursor_monitor(&window);
            let _ = window.show();
            let _ = window.set_focus();
            ensure_overlay(&window);
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
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut_pressed, event| {
                    if shortcut_pressed == &shortcut && event.state == ShortcutState::Pressed {
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
                        if let Some(window) = app.get_webview_window("main") {
                            reposition_to_cursor_monitor(&window);
                            let _ = window.show();
                            let _ = window.set_focus();
                            ensure_overlay(&window);
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

            // Configure window size and position on the cursor's monitor
            if let Some(window) = app.get_webview_window("main") {
                reposition_to_cursor_monitor(&window);

                // Apply vibrancy effect on macOS
                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                    let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(16.0));
                    println!("Applied vibrancy effect");
                }

                // Show window and apply overlay properties (level, collection behavior)
                let _ = window.show();
                ensure_overlay(&window);
            }

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
            prepare_image_for_drag,
            create_temp_text_file,
            create_temp_link_file,
            create_drag_icon,
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
            // Window commands
            hide_window,
            show_window,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

