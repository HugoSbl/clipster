use tauri::{AppHandle, Manager};

/// Reposition the window to the bottom of the monitor where the cursor is.
/// Called every time the window is shown so it follows the user across screens.
#[cfg(target_os = "macos")]
pub fn reposition_to_cursor_monitor(window: &tauri::WebviewWindow) {
    // Use CoreGraphics C functions directly — avoids objc2 msg_send Encode issues
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGSize {
        width: f64,
        height: f64,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    type CGDirectDisplayID = u32;

    extern "C" {
        fn CGEventCreate(source: *const std::ffi::c_void) -> *const std::ffi::c_void;
        fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
        fn CFRelease(cf: *const std::ffi::c_void);
        fn CGGetActiveDisplayList(
            max: u32,
            displays: *mut CGDirectDisplayID,
            count: *mut u32,
        ) -> i32;
        fn CGDisplayBounds(display: CGDirectDisplayID) -> CGRect;
    }

    unsafe {
        // Cursor position in global display coords (top-left origin)
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            return;
        }
        let cursor = CGEventGetLocation(event);
        CFRelease(event);

        // Enumerate active displays
        let mut display_count: u32 = 0;
        if CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut display_count) != 0
            || display_count == 0
        {
            return;
        }
        let mut displays = vec![0u32; display_count as usize];
        if CGGetActiveDisplayList(display_count, displays.as_mut_ptr(), &mut display_count) != 0 {
            return;
        }

        // Find display containing cursor
        for &display_id in &displays {
            let bounds = CGDisplayBounds(display_id);

            if cursor.x >= bounds.origin.x
                && cursor.x < bounds.origin.x + bounds.size.width
                && cursor.y >= bounds.origin.y
                && cursor.y < bounds.origin.y + bounds.size.height
            {
                let win_h = bounds.size.height * 0.33;
                // Global display coords use top-left origin, same as Tauri
                let win_y = bounds.origin.y + bounds.size.height - win_h;

                let _ = window.set_size(tauri::LogicalSize::new(bounds.size.width, win_h));
                let _ = window.set_position(tauri::LogicalPosition::new(bounds.origin.x, win_y));
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
