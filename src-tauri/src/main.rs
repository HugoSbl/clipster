// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod models;
mod storage;

use commands::clipboard_commands::{
    assign_to_pinboard, clear_clipboard_history, copy_to_clipboard, delete_clipboard_item,
    get_clipboard, get_clipboard_count, get_clipboard_history, get_clipboard_item,
    get_image_data, prepare_image_for_drag, search_clipboard, toggle_favorite,
};
use commands::pinboard_commands::{
    add_item_to_pinboard, create_pinboard, delete_pinboard, get_pinboard, get_pinboard_items,
    get_pinboards, remove_item_from_pinboard, reorder_pinboards, update_pinboard,
};
use commands::settings_commands::{
    get_history_limit, get_settings, set_history_limit, update_setting,
};
use commands::window_commands::{hide_window, show_window};
use std::sync::Arc;
use storage::Database;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use clipboard::clipboard_monitor;

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
        .plugin(tauri_plugin_drag::init())
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

            // Configure window size and position based on primary monitor
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let size = monitor.size();
                    let scale = monitor.scale_factor();

                    // Calculate dimensions: full width, 1/3 height
                    let width = size.width as f64 / scale;
                    let height = (size.height as f64 / scale) * 0.33;

                    // Position at bottom of screen (above dock area)
                    let y = (size.height as f64 / scale) - height;

                    // Apply size and position
                    if let Err(e) = window.set_size(tauri::LogicalSize::new(width, height)) {
                        eprintln!("Failed to set window size: {}", e);
                    }
                    if let Err(e) = window.set_position(tauri::LogicalPosition::new(0.0, y)) {
                        eprintln!("Failed to set window position: {}", e);
                    }

                    println!("Window configured: {}x{} at y={}", width, height, y);
                } else {
                    eprintln!("Could not get primary monitor");
                }

                // Apply vibrancy effect and window level on macOS
                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

                    // Apply vibrancy
                    let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(16.0));
                    println!("Applied vibrancy effect");

                    // Set window level and collection behavior for full-screen support
                    if let Ok(ns_window) = window.ns_window() {
                        unsafe {
                            use objc2::runtime::AnyObject;
                            use objc2::msg_send;
                            let ns_win: *mut AnyObject = ns_window as *mut AnyObject;

                            // kCGDockWindowLevel = 20, we use 25 to be above it
                            let _: () = msg_send![ns_win, setLevel: 25_i64];
                            println!("Window level set above dock");

                            // Set collection behavior to appear on all spaces including full-screen
                            // NSWindowCollectionBehaviorCanJoinAllSpaces = 1 << 0
                            // NSWindowCollectionBehaviorFullScreenAuxiliary = 1 << 8
                            // NSWindowCollectionBehaviorStationary = 1 << 4 (stays in place during Mission Control)
                            let behavior: u64 = (1 << 0) | (1 << 8) | (1 << 4);
                            let _: () = msg_send![ns_win, setCollectionBehavior: behavior];
                            println!("Window collection behavior set for all spaces");
                        }
                    }
                }

                // Show window after configuration
                let _ = window.show();
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

