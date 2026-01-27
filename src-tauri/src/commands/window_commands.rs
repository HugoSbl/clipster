use tauri::{AppHandle, Manager};

/// Reposition the window to the bottom of the monitor where the cursor is.
/// Called every time the window is shown so it follows the user across screens.
#[cfg(target_os = "macos")]
pub fn reposition_to_cursor_monitor(window: &tauri::WebviewWindow) {
    use objc2::encode::{Encode, Encoding};

    // Minimal Cocoa geometry types for msg_send — must implement Encode
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    unsafe impl Encode for CGPoint {
        const ENCODING: Encoding =
            Encoding::Struct("CGPoint", &[Encoding::Double, Encoding::Double]);
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGSize {
        width: f64,
        height: f64,
    }
    unsafe impl Encode for CGSize {
        const ENCODING: Encoding =
            Encoding::Struct("CGSize", &[Encoding::Double, Encoding::Double]);
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }
    unsafe impl Encode for CGRect {
        const ENCODING: Encoding = Encoding::Struct(
            "CGRect",
            &[
                Encoding::Struct("CGPoint", &[Encoding::Double, Encoding::Double]),
                Encoding::Struct("CGSize", &[Encoding::Double, Encoding::Double]),
            ],
        );
    }

    unsafe {
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject};

        // [NSEvent mouseLocation] — cursor position in Cocoa coords (bottom-left origin)
        let event_class = match AnyClass::get("NSEvent") {
            Some(c) => c,
            None => return,
        };
        let cursor: CGPoint = msg_send![event_class, mouseLocation];

        // [NSScreen screens] — ordered list, index 0 = primary
        let screen_class = match AnyClass::get("NSScreen") {
            Some(c) => c,
            None => return,
        };
        let screens: *const AnyObject = msg_send![screen_class, screens];
        let count: usize = msg_send![screens, count];
        if count == 0 {
            return;
        }

        // Primary screen height for Cocoa→Tauri coordinate conversion
        let primary: *const AnyObject = msg_send![screens, objectAtIndex: 0_usize];
        let primary_frame: CGRect = msg_send![primary, frame];
        let primary_h = primary_frame.size.height;

        for i in 0..count {
            let screen: *const AnyObject = msg_send![screens, objectAtIndex: i];
            let frame: CGRect = msg_send![screen, frame];

            // Check if cursor is on this screen
            if cursor.x >= frame.origin.x
                && cursor.x < frame.origin.x + frame.size.width
                && cursor.y >= frame.origin.y
                && cursor.y < frame.origin.y + frame.size.height
            {
                let win_h = frame.size.height * 0.33;
                // Cocoa y (bottom-left) → Tauri y (top-left):
                // window bottom = screen bottom → tauri_y = primary_h - screen.origin.y - win_h
                let tauri_y = primary_h - frame.origin.y - win_h;

                let _ = window.set_size(tauri::LogicalSize::new(frame.size.width, win_h));
                let _ =
                    window.set_position(tauri::LogicalPosition::new(frame.origin.x, tauri_y));
                return;
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn reposition_to_cursor_monitor(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut point);

        let hmonitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let _ = GetMonitorInfoW(hmonitor, &mut info);

        let rc = info.rcMonitor;
        let phys_w = (rc.right - rc.left) as f64;
        let phys_h = (rc.bottom - rc.top) as f64;
        let phys_x = rc.left as f64;
        let phys_y = rc.top as f64;

        // Convert physical pixels → logical points
        let scale = window.scale_factor().unwrap_or(1.0);
        let w = phys_w / scale;
        let h = phys_h / scale;
        let x = phys_x / scale;
        let y = phys_y / scale;

        let win_h = h * 0.33;
        let win_y = y + h - win_h;

        let _ = window.set_size(tauri::LogicalSize::new(w, win_h));
        let _ = window.set_position(tauri::LogicalPosition::new(x, win_y));
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn reposition_to_cursor_monitor(_window: &tauri::WebviewWindow) {}

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
        reposition_to_cursor_monitor(&window);
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
