use tauri::{AppHandle, Manager};

// ── macOS native helpers ──────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
use std::sync::Once;
#[cfg(target_os = "macos")]
use objc2::msg_send;
#[cfg(target_os = "macos")]
use objc2::runtime::{AnyClass, AnyObject};
#[cfg(target_os = "macos")]
use objc2_foundation::{CGPoint, CGRect, CGSize};

/// Link to the Objective-C runtime.
#[cfg(target_os = "macos")]
#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn object_setClass(
        obj: *mut std::ffi::c_void,
        cls: *const std::ffi::c_void,
    ) -> *const std::ffi::c_void;
    fn object_getClassName(obj: *const std::ffi::c_void) -> *const std::ffi::c_char;
    fn objc_allocateClassPair(
        superclass: *const std::ffi::c_void,
        name: *const std::ffi::c_char,
        extra_bytes: usize,
    ) -> *mut std::ffi::c_void;
    fn objc_registerClassPair(cls: *mut std::ffi::c_void);
    fn class_addMethod(
        cls: *mut std::ffi::c_void,
        sel: *const std::ffi::c_void,
        imp: *const std::ffi::c_void,
        types: *const std::ffi::c_char,
    ) -> bool;
    fn sel_registerName(name: *const std::ffi::c_char) -> *const std::ffi::c_void;
}

// ── SpotlightPanel: NSPanel subclass with canBecomeKeyWindow = YES ───────────
//
// A borderless NSPanel with NonactivatingPanel defaults canBecomeKeyWindow to
// NO.  WebKit interprets this as "background window" and suppresses mouse
// tracking (hover, cursor changes).  We define a runtime subclass that
// overrides canBecomeKeyWindow → YES so WebKit treats it as fully interactive.
//
// Uses raw ObjC runtime C API to avoid objc2 version conflicts (Tauri pulls
// a different objc2 version internally, making ClassBuilder::add_method fail
// with MethodImplementation trait mismatch).

#[cfg(target_os = "macos")]
static REGISTER_SPOTLIGHT_PANEL: Once = Once::new();

/// IMP for -[SpotlightPanel canBecomeKeyWindow] → YES
#[cfg(target_os = "macos")]
extern "C" fn spotlight_panel_can_become_key(
    _self: *mut std::ffi::c_void,
    _cmd: *mut std::ffi::c_void,
) -> bool {
    true
}

/// Register the `SpotlightPanel` ObjC class (once).  Must be called before
/// the first `object_setClass` that uses it.
#[cfg(target_os = "macos")]
fn register_spotlight_panel_class() {
    REGISTER_SPOTLIGHT_PANEL.call_once(|| {
        unsafe {
            let superclass = AnyClass::get("NSPanel").expect("NSPanel class not found");

            let cls = objc_allocateClassPair(
                superclass as *const AnyClass as *const std::ffi::c_void,
                b"SpotlightPanel\0".as_ptr() as *const std::ffi::c_char,
                0,
            );
            assert!(!cls.is_null(), "Failed to allocate SpotlightPanel class");

            let sel = sel_registerName(
                b"canBecomeKeyWindow\0".as_ptr() as *const std::ffi::c_char,
            );
            // Type encoding: returns BOOL (B on arm64, c on x86_64), self (@), _cmd (:)
            #[cfg(target_arch = "aarch64")]
            let types = b"B@:\0";
            #[cfg(target_arch = "x86_64")]
            let types = b"c@:\0";

            class_addMethod(
                cls,
                sel,
                spotlight_panel_can_become_key as *const std::ffi::c_void,
                types.as_ptr() as *const std::ffi::c_char,
            );

            objc_registerClassPair(cls);
            println!("register_spotlight_panel_class: registered SpotlightPanel");
        }
    });
}

/// Return the shared `NSApplication` instance.
#[cfg(target_os = "macos")]
unsafe fn ns_app() -> *mut AnyObject {
    let cls = AnyClass::get("NSApplication").unwrap();
    msg_send![cls, sharedApplication]
}

/// Return the raw `NSWindow` pointer behind a Tauri window.
#[cfg(target_os = "macos")]
fn ns_window_ptr(window: &tauri::WebviewWindow) -> Option<*mut AnyObject> {
    window.ns_window().ok().map(|w| w as *mut AnyObject)
}

// ── Reposition to cursor monitor ──────────────────────────────────────────────

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

// ── NSPanel runtime-swizzle pattern ───────────────────────────────────────────
//
// Standard NSWindow approaches (setLevel, setCollectionBehavior, etc.) all fail
// to overlay fullscreen apps because macOS treats NSWindow and NSPanel
// differently at the window-server level.  NSPanel with the
// NonactivatingPanel style mask is the mechanism Spotlight/Raycast use.
//
// Since Tauri creates an NSWindow, we swap the isa pointer at runtime via
// object_setClass(obj, [NSPanel class]) so the window server treats it as a
// true panel.  Combined with LSUIElement (Info.plist) and the correct
// collection behaviors, the panel appears over ANY app including fullscreen.
//
// CRITICAL: hide_panel must NOT call [NSApp hide:].  That destroys the
// window's Space affinity and prevents it from reappearing over fullscreen
// apps on the next show.
//
// NonactivatingPanel suppresses mouse tracking (hover, cursor changes) by
// default.  We fix this by injecting NSTrackingAreas with ActiveAlways +
// CursorUpdate on every view in the hierarchy (the WKWebView is nested
// several levels deep).

// ── Recursive NSTrackingArea injection ────────────────────────────────────────

/// Attach an `NSTrackingArea` to `view` and recurse into all its subviews.
/// Returns the total number of views patched.
#[cfg(target_os = "macos")]
unsafe fn apply_tracking_recursive(view: *mut AnyObject) -> usize {
    if view.is_null() {
        return 0;
    }

    let mut count = 0;

    // ── Attach a tracking area to this view ──────────────────────────────
    add_tracking_area_to_view(view);
    count += 1;

    let cls_name =
        std::ffi::CStr::from_ptr(object_getClassName(view as *const std::ffi::c_void));
    println!("  patched view: {:?}", cls_name);

    // ── Recurse into subviews ────────────────────────────────────────────
    let subviews: *mut AnyObject = msg_send![view, subviews];
    if !subviews.is_null() {
        let len: usize = msg_send![subviews, count];
        for i in 0..len {
            let child: *mut AnyObject = msg_send![subviews, objectAtIndex: i];
            count += apply_tracking_recursive(child);
        }
    }

    count
}

/// Create and add a single `NSTrackingArea` to `view`.
#[cfg(target_os = "macos")]
unsafe fn add_tracking_area_to_view(view: *mut AnyObject) {
    //   NSTrackingMouseEnteredAndExited = 0x01
    //   NSTrackingMouseMoved            = 0x02   ← drives :hover
    //   NSTrackingCursorUpdate          = 0x04   ← drives cursor: pointer/grab
    //   NSTrackingActiveAlways          = 0x80   ← track even when panel is inactive
    //   NSTrackingInVisibleRect         = 0x200  ← auto-resize with view bounds
    //   NSTrackingEnabledDuringMouseDrag= 0x400  ← keep tracking while dragging
    let options: usize = 0x01 | 0x02 | 0x04 | 0x80 | 0x200 | 0x400;

    // Rect is ignored when InVisibleRect is set
    let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(0.0, 0.0));
    let nil: *mut AnyObject = std::ptr::null_mut();

    let tracking_cls =
        AnyClass::get("NSTrackingArea").expect("NSTrackingArea class not found");
    let tracking_area: *mut AnyObject = msg_send![tracking_cls, alloc];
    let tracking_area: *mut AnyObject = msg_send![
        tracking_area,
        initWithRect: rect
        options: options
        owner: view
        userInfo: nil
    ];

    let _: () = msg_send![view, addTrackingArea: tracking_area];
}

// ── NSPanel runtime-swizzle + tracking ───────────────────────────────────────

/// One-time setup: swizzle the Tauri NSWindow into an NSPanel and apply
/// the NonactivatingPanel style mask.
///
/// Call this once during `setup`, **before** the first `show_panel`.
pub fn setup_window_behavior(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let Some(ns_win) = ns_window_ptr(window) else {
            eprintln!("setup_window_behavior: could not obtain NSWindow");
            return;
        };
        unsafe {
            // ── 1. Swizzle NSWindow → SpotlightPanel ─────────────────────
            // SpotlightPanel is a runtime subclass of NSPanel that overrides
            // canBecomeKeyWindow → YES.  Without this, WebKit treats the
            // borderless NonactivatingPanel as a background window and
            // suppresses :hover / cursor CSS states.
            register_spotlight_panel_class();

            let before = std::ffi::CStr::from_ptr(
                object_getClassName(ns_win as *const std::ffi::c_void),
            );
            println!("setup_window_behavior: class BEFORE = {:?}", before);

            let panel_cls = AnyClass::get("SpotlightPanel")
                .expect("SpotlightPanel class not found — register_spotlight_panel_class failed");
            object_setClass(
                ns_win as *mut std::ffi::c_void,
                panel_cls as *const AnyClass as *const std::ffi::c_void,
            );

            let after = std::ffi::CStr::from_ptr(
                object_getClassName(ns_win as *const std::ffi::c_void),
            );
            println!("setup_window_behavior: class AFTER  = {:?}", after);

            // ── 2. Style mask ─────────────────────────────────────────────
            // Preserve existing bits (e.g. FullSizeContentView), strip
            // Titled, add NonactivatingPanel + Resizable.
            //   NSWindowStyleMaskTitled              = 1 << 0
            //   NSWindowStyleMaskResizable           = 1 << 3
            //   NSWindowStyleMaskNonactivatingPanel  = 1 << 7
            let current_mask: u64 = msg_send![ns_win, styleMask];
            let new_mask = (current_mask & !(1_u64))  // remove Titled
                | (1_u64 << 7)                        // NonactivatingPanel
                | (1_u64 << 3);                       // Resizable
            let _: () = msg_send![ns_win, setStyleMask: new_mask];
            println!(
                "setup_window_behavior: styleMask 0x{:X} -> 0x{:X}",
                current_mask, new_mask
            );

            // ── 3. Panel properties ───────────────────────────────────────
            let _: () = msg_send![ns_win, setHidesOnDeactivate: false];
            let _: () = msg_send![ns_win, setHasShadow: false];

            // ── 4. Mouse tracking ────────────────────────────────────────
            // NonactivatingPanel suppresses hover/cursor events by default.
            // Inject NSTrackingAreas on the contentView AND every subview
            // recursively (the WKWebView is nested several levels deep).
            let _: () = msg_send![ns_win, setAcceptsMouseMovedEvents: true];
            let _: () = msg_send![ns_win, setFloatingPanel: true];

            let content_view: *mut AnyObject = msg_send![ns_win, contentView];
            if !content_view.is_null() {
                let count = apply_tracking_recursive(content_view);
                println!(
                    "setup_window_behavior: NSTrackingArea injected on {} views",
                    count
                );
            }
        }
    }
}

/// Show the panel over any app (including fullscreen) and grab keyboard focus.
///
/// Re-applies collection behavior and level on EVERY show because orderOut
/// can reset them.  Debug-prints the actual class + level after ordering
/// front so you can verify the swizzle is intact.
pub fn show_panel(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let Some(ns_win) = ns_window_ptr(window) else {
            eprintln!("show_panel: could not obtain NSWindow");
            return;
        };
        unsafe {
            let nil: *mut AnyObject = std::ptr::null_mut();

            // ── 1. Collection behavior (force every show) ─────────────────
            //   CanJoinAllSpaces       (1 << 0)
            //   Stationary             (1 << 4)
            //   IgnoresCycle           (1 << 6)
            //   FullScreenAuxiliary    (1 << 8)
            let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 6) | (1 << 8);
            let _: () = msg_send![ns_win, setCollectionBehavior: behavior];

            // ── 2. Level: kCGStatusWindowLevel (25) ───────────────────────
            let _: () = msg_send![ns_win, setLevel: 25_i64];

            // ── 3. Show + accept keyboard ─────────────────────────────────
            let _: () = msg_send![ns_win, makeKeyAndOrderFront: nil];

            // ── 4. Activate for keyboard routing ──────────────────────────
            let app = ns_app();
            let _: () = msg_send![app, activateIgnoringOtherApps: true];

            // ── Debug: verify swizzle + level stuck ───────────────────────
            let actual_level: i64 = msg_send![ns_win, level];
            let cls_name = std::ffi::CStr::from_ptr(
                object_getClassName(ns_win as *const std::ffi::c_void),
            );
            println!(
                "show_panel: class={:?} level={} behavior=0x{:X}",
                cls_name, actual_level, behavior
            );
        }
    }

    #[cfg(target_os = "windows")]
    {
        let _ = window.set_always_on_top(true);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hide the panel.  ONLY orderOut — do NOT call [NSApp hide:].
///
/// [NSApp hide:] destroys the window's Space affinity, preventing it from
/// reappearing over fullscreen apps on the next show_panel call.
pub fn hide_panel(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let Some(ns_win) = ns_window_ptr(window) else {
            eprintln!("hide_panel: could not obtain NSWindow");
            return;
        };
        unsafe {
            let nil: *mut AnyObject = std::ptr::null_mut();
            let _: () = msg_send![ns_win, orderOut: nil];
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.hide();
    }
}

// ── Tauri IPC commands ────────────────────────────────────────────────────────

/// Hide the main window
#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        hide_panel(&window);
    }
    Ok(())
}

/// Show the main window
#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        reposition_to_cursor_monitor(&window);
        show_panel(&window);
    }
    Ok(())
}

/// Quit the application
#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}
