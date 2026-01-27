use tauri::{AppHandle, Manager};

/// Ensure the window appears above everything, including fullscreen apps.
/// Must be called every time the window is shown (macOS can reset these).
pub fn ensure_overlay(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        if let Ok(ns_window) = window.ns_window() {
            unsafe {
                use objc2::msg_send;
                use objc2::runtime::AnyObject;
                let ns_win: *mut AnyObject = ns_window as *mut AnyObject;

                // kCGScreenSaverWindowLevel (1000) — above fullscreen apps
                let _: () = msg_send![ns_win, setLevel: 1000_i64];

                // NSWindowCollectionBehaviorCanJoinAllSpaces  (1 << 0)  — all Spaces
                // NSWindowCollectionBehaviorStationary        (1 << 4)  — stays during Mission Control
                // NSWindowCollectionBehaviorIgnoresCycle      (1 << 6)  — skip Cmd+Tab
                // NSWindowCollectionBehaviorFullScreenAuxiliary (1 << 8) — overlay fullscreen Spaces
                let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 6) | (1 << 8);
                let _: () = msg_send![ns_win, setCollectionBehavior: behavior];
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Re-assert topmost so it stays above fullscreen (borderless) apps
        let _ = window.set_always_on_top(true);
    }
}

/// Hide the main window
#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Show the main window
#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        ensure_overlay(&window);
    }
    Ok(())
}

/// Quit the application
#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}
