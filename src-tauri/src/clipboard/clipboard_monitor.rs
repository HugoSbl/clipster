//! Cross-platform clipboard monitoring
//!
//! Windows: Uses clipboard-master crate for native clipboard notifications
//! macOS: Uses polling with arboard

use crate::clipboard::clipboard_reader::{self, ClipboardContent};
use crate::models::ClipboardItem;
use crate::storage::{file_storage, Database, FileStorage};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter};

/// Global monitor instance
static MONITOR_HANDLE: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();
static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
static LAST_IMAGE_HASH: AtomicU64 = AtomicU64::new(0);

/// Event payload for clipboard changes
#[derive(Clone, serde::Serialize)]
pub struct ClipboardChangedPayload {
    pub item: ClipboardItem,
    /// If this item replaced an existing one (move to top), this contains the old item's ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_item_id: Option<String>,
}

/// Clipboard handler that processes clipboard changes
struct ClipboardMonitorHandler {
    app_handle: AppHandle,
    db: Arc<Database>,
    file_storage: FileStorage,
}

impl ClipboardMonitorHandler {
    fn new(app_handle: AppHandle, db: Arc<Database>) -> Self {
        let file_storage = FileStorage::new().expect("Failed to initialize file storage");
        Self {
            app_handle,
            db,
            file_storage,
        }
    }

    /// Process new clipboard content
    fn process_clipboard_change(&self) {
        let content = clipboard_reader::read_clipboard();

        match content {
            ClipboardContent::Text(text) => self.process_text(text),
            ClipboardContent::Image(image_data) => self.process_image(image_data),
            ClipboardContent::Files(files) => self.process_files(files),
            ClipboardContent::Empty => {}
        }
    }

    /// Process text clipboard content
    /// Uses "move to top" behavior: if content exists, delete old and create new
    /// Pinned items are preserved - only unpinned history items are affected
    fn process_text(&self, text: String) {
        eprintln!("╔═══════════════════════════════════════════════════════════");
        eprintln!("║ [DEBUG process_text] NEW TEXT FROM CLIPBOARD");
        let preview = if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() };
        eprintln!("║   text: {} ({} chars)", preview.replace('\n', "\\n"), text.len());

        if text.trim().is_empty() {
            eprintln!("║   EMPTY/WHITESPACE - skipping");
            eprintln!("╚═══════════════════════════════════════════════════════════");
            return;
        }

        // "Move to top" behavior: delete existing unpinned item, then create new
        // This ensures the most recent copy is always at the top
        // Pinned items are NOT affected - they stay in their pinboards
        let (replaced_item_id, original_source_app, original_source_icon) =
            match self.db.delete_unpinned_by_content(&text) {
                Ok(Some((id, app, icon))) => {
                    eprintln!("║   MOVE TO TOP: deleted existing item {} (app: {:?})", id, app);
                    (Some(id), app, icon)
                }
                Ok(None) => {
                    eprintln!("║   New content (not in unpinned history)");
                    (None, None, None)
                }
                Err(e) => {
                    eprintln!("║   Warning: delete_unpinned_by_content failed: {}", e);
                    (None, None, None)
                }
            };

        // Preserve original source app icon when re-copying from within the app
        let (source_app, source_app_icon) = if original_source_app.is_some() {
            (original_source_app, original_source_icon)
        } else {
            self.get_source_app_info()
        };
        eprintln!("║   source_app: {:?}", source_app);
        eprintln!("╚═══════════════════════════════════════════════════════════");

        let item = ClipboardItem::new_text(text, source_app, source_app_icon);
        self.save_and_emit(item, replaced_item_id);
    }

    /// Calculate hash of bytes for deduplication
    fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Process image clipboard content
    /// CRITICAL: This function MUST NEVER silently drop an image.
    /// Even if decoding fails, we save the raw PNG bytes.
    fn process_image(&self, image_data: clipboard_reader::ImageData) {
        eprintln!("╔═══════════════════════════════════════════════════════════");
        eprintln!("║ [DEBUG process_image] NEW IMAGE FROM CLIPBOARD");
        eprintln!("║   png_data size: {} bytes", image_data.png_data.len());

        // Deduplicate images using hash of first 10KB
        let hash_len = std::cmp::min(10240, image_data.png_data.len());
        let image_hash = Self::hash_bytes(&image_data.png_data[..hash_len]);

        eprintln!("║   hash (first {}B): {}", hash_len, image_hash);

        if image_hash == LAST_IMAGE_HASH.load(Ordering::SeqCst) {
            eprintln!("║   DUPLICATE - skipping");
            eprintln!("╚═══════════════════════════════════════════════════════════");
            return;
        }
        LAST_IMAGE_HASH.store(image_hash, Ordering::SeqCst);

        let id = uuid::Uuid::new_v4().to_string();
        eprintln!("║   Generated UUID: {}", id);

        // Try to decode image - but DON'T fail if this doesn't work
        match image::load_from_memory(&image_data.png_data) {
            Ok(image) => {
                eprintln!("║   Image decoded: {}x{}", image.width(), image.height());

                // Generate thumbnail (continue even if this fails)
                let thumbnail_base64 = match file_storage::generate_thumbnail_default(&image) {
                    Ok(png_bytes) => {
                        eprintln!("║   Thumbnail generated: {} bytes", png_bytes.len());
                        Some(file_storage::thumbnail_to_base64(&png_bytes))
                    }
                    Err(e) => {
                        eprintln!("║   Thumbnail FAILED: {} (continuing with placeholder)", e);
                        None
                    }
                };

                // Save full image to disk
                let image_path = match self.file_storage.save_image(&id, &image) {
                    Ok(path) => {
                        let path_str = path.to_string_lossy().to_string();
                        // Verify the saved file
                        if let Ok(meta) = std::fs::metadata(&path) {
                            eprintln!("║   Image SAVED: {}", path_str);
                            eprintln!("║   Saved file size: {} bytes", meta.len());
                        }
                        path_str
                    }
                    Err(e) => {
                        eprintln!("║   Image save via image crate FAILED: {}", e);
                        // FALLBACK: Save raw PNG bytes directly
                        self.save_raw_png_and_emit(&id, &image_data.png_data, None);
                        return;
                    }
                };

                let (source_app, source_app_icon) = self.get_source_app_info();
                eprintln!("║   source_app: {:?}", source_app);
                eprintln!("╚═══════════════════════════════════════════════════════════");

                let item =
                    ClipboardItem::new_image(thumbnail_base64, image_path, source_app, source_app_icon);
                // Images use hash-based deduplication, not "move to top"
                self.save_and_emit(item, None);
            }
            Err(e) => {
                eprintln!("║   Image decode FAILED: {}", e);
                eprintln!("║   FALLBACK: Saving raw PNG bytes directly...");
                // CRITICAL FALLBACK: Even if we can't decode the image, save the raw bytes
                // This ensures NO clipboard capture is ever lost
                self.save_raw_png_and_emit(&id, &image_data.png_data, Some(e.to_string()));
            }
        }
    }

    /// Fallback: Save raw PNG bytes when image decoding fails
    /// This ensures we NEVER lose a clipboard capture
    fn save_raw_png_and_emit(&self, id: &str, png_data: &[u8], decode_error: Option<String>) {
        eprintln!("║   [FALLBACK] Saving raw PNG ({} bytes)...", png_data.len());

        // Try to save raw PNG bytes to disk
        let image_path = match self.file_storage.save_png_bytes(id, png_data) {
            Ok(path) => {
                let path_str = path.to_string_lossy().to_string();
                eprintln!("║   [FALLBACK] Raw PNG saved: {}", path_str);
                path_str
            }
            Err(e) => {
                eprintln!("║   [FALLBACK] CRITICAL: Even raw save failed: {}", e);
                eprintln!("║   ITEM LOST - this should never happen!");
                eprintln!("╚═══════════════════════════════════════════════════════════");
                return;
            }
        };

        // Create thumbnail from raw data (might work even if full decode failed)
        let thumbnail_base64 = image::load_from_memory(png_data)
            .ok()
            .and_then(|img| file_storage::generate_thumbnail_default(&img).ok())
            .map(|bytes| file_storage::thumbnail_to_base64(&bytes));

        let (source_app, source_app_icon) = self.get_source_app_info();

        if let Some(err) = decode_error {
            eprintln!("║   [FALLBACK] Original decode error: {}", err);
        }
        eprintln!("║   [FALLBACK] source_app: {:?}", source_app);
        eprintln!("╚═══════════════════════════════════════════════════════════");

        let item = ClipboardItem::new_image(thumbnail_base64, image_path, source_app, source_app_icon);
        // Images use hash-based deduplication, not "move to top"
        self.save_and_emit(item, None);
    }

    /// Process file list clipboard content
    /// CRITICAL: This function MUST NEVER silently drop files.
    /// Even if thumbnail generation fails, we still save the item.
    /// Uses "move to top" behavior for duplicates.
    fn process_files(&self, files: Vec<String>) {
        eprintln!("╔═══════════════════════════════════════════════════════════");
        eprintln!("║ [DEBUG process_files] Processing {} files", files.len());
        for (i, f) in files.iter().enumerate() {
            let exists = std::path::Path::new(f).exists();
            eprintln!("║   [{}] {} (exists: {})", i, f, exists);
        }

        if files.is_empty() {
            eprintln!("║   EMPTY - skipping");
            eprintln!("╚═══════════════════════════════════════════════════════════");
            return;
        }

        // "Move to top" behavior: delete existing unpinned item, then create new
        let files_json = serde_json::to_string(&files).unwrap_or_default();

        let (replaced_item_id, original_source_app, original_source_icon) =
            match self.db.delete_unpinned_by_content(&files_json) {
                Ok(Some((id, app, icon))) => {
                    eprintln!("║   MOVE TO TOP: deleted existing item {} (app: {:?})", id, app);
                    (Some(id), app, icon)
                }
                Ok(None) => {
                    eprintln!("║   New content (not in unpinned history)");
                    (None, None, None)
                }
                Err(e) => {
                    eprintln!("║   Warning: delete_unpinned_by_content failed: {}", e);
                    (None, None, None)
                }
            };

        // Generate thumbnail for the first file (if possible)
        // IMPORTANT: Thumbnail failure MUST NOT prevent item creation
        eprintln!("║   Generating thumbnail (failure OK)...");
        let thumbnail_base64 = self.generate_file_thumbnail(&files);
        match &thumbnail_base64 {
            Some(t) => eprintln!("║   Thumbnail: {} chars", t.len()),
            None => eprintln!("║   Thumbnail: None (will use file icon)"),
        }

        // Preserve original source app icon when re-copying from within the app
        let (source_app, source_app_icon) = if original_source_app.is_some() {
            (original_source_app, original_source_icon)
        } else {
            // For new files, use the file's own icon instead of source app
            // This is more informative (shows PDF icon, Word icon, etc.)
            let first_file = &files[0];
            self.get_file_app_info(first_file)
        };
        eprintln!("║   source_app: {:?}", source_app);
        eprintln!("╚═══════════════════════════════════════════════════════════");

        let item = ClipboardItem::new_files_with_thumbnail(
            files,
            source_app,
            source_app_icon,
            thumbnail_base64,
        );
        self.save_and_emit(item, replaced_item_id);
    }

    /// Get file type icon and app name for a file path
    #[cfg(target_os = "macos")]
    fn get_file_app_info(&self, file_path: &str) -> (Option<String>, Option<String>) {
        use base64::Engine;
        use objc2::ClassType;
        use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSImage, NSWorkspace};
        use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};
        use std::path::Path;

        let path = Path::new(file_path);

        // Get the file extension for a descriptive name
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_uppercase())
            .unwrap_or_else(|| "File".to_string());

        // Get the file icon using NSWorkspace
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let ns_path = NSString::from_str(file_path);
            let icon = workspace.iconForFile(&ns_path);

            // Resize to 32x32 and convert to PNG
            let target_size = NSSize::new(32.0, 32.0);
            let resized_image = NSImage::initWithSize(NSImage::alloc(), target_size);

            #[allow(deprecated)]
            resized_image.lockFocus();

            let source_rect = NSRect::new(NSPoint::new(0.0, 0.0), icon.size());
            let dest_rect = NSRect::new(NSPoint::new(0.0, 0.0), target_size);

            icon.drawInRect_fromRect_operation_fraction(
                dest_rect,
                source_rect,
                NSCompositingOperation::SourceOver,
                1.0,
            );

            #[allow(deprecated)]
            resized_image.unlockFocus();

            // Convert to PNG
            if let Some(tiff_data) = resized_image.TIFFRepresentation() {
                if let Some(bitmap_rep) = NSBitmapImageRep::imageRepWithData(&tiff_data) {
                    if let Some(png_data) = bitmap_rep.representationUsingType_properties(
                        NSBitmapImageFileType::PNG,
                        &NSDictionary::new(),
                    ) {
                        let bytes = png_data.bytes();
                        let base64_str = base64::engine::general_purpose::STANDARD.encode(bytes);
                        return (Some(extension), Some(base64_str));
                    }
                }
            }
        }

        (Some(extension), None)
    }

    #[cfg(target_os = "windows")]
    fn get_file_app_info(&self, file_path: &str) -> (Option<String>, Option<String>) {
        use std::path::Path;

        let path = Path::new(file_path);
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_uppercase())
            .unwrap_or_else(|| "File".to_string());

        // Use the existing file icon extraction for Windows
        let icon = file_storage::generate_file_thumbnail_windows(path, 32)
            .map(|bytes| file_storage::thumbnail_to_base64(&bytes));

        (Some(extension), icon)
    }

    /// Generate a thumbnail for the first file in the list
    fn generate_file_thumbnail(&self, files: &[String]) -> Option<String> {
        if files.is_empty() {
            return None;
        }

        let first_file = &files[0];
        let path = std::path::Path::new(first_file);

        // Use platform-specific thumbnail generation
        #[cfg(target_os = "macos")]
        let thumbnail_bytes = file_storage::generate_file_thumbnail_macos(path, 400)?;

        #[cfg(target_os = "windows")]
        let thumbnail_bytes = file_storage::generate_file_thumbnail_windows(path, 400)?;

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let thumbnail_bytes: Option<Vec<u8>> = None;

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let _ = thumbnail_bytes?;

        // Check thumbnail size (skip if too large, > 300KB)
        let size_kb = thumbnail_bytes.len() / 1024;
        if thumbnail_bytes.len() > 300 * 1024 {
            eprintln!("[generate_file_thumbnail] Thumbnail too large: {}KB > 300KB, skipping", size_kb);
            return None;
        }
        eprintln!("[generate_file_thumbnail] Thumbnail size OK: {}KB", size_kb);

        Some(file_storage::thumbnail_to_base64(&thumbnail_bytes))
    }

    /// Save item to database and emit event to frontend
    /// CRITICAL: This is the final step - if this fails, the item is lost
    /// replaced_item_id: If this item replaced an existing one (move to top), pass the old ID
    fn save_and_emit(&self, item: ClipboardItem, replaced_item_id: Option<String>) {
        eprintln!("╔═══════════════════════════════════════════════════════════");
        eprintln!("║ [save_and_emit] SAVING TO DATABASE");
        eprintln!("║   id: {}", item.id);
        eprintln!("║   content_type: {:?}", item.content_type);
        eprintln!("║   content_text: {:?}", item.content_text.as_ref().map(|s| {
            if s.len() > 50 { format!("{}... ({} chars)", &s[..50], s.len()) } else { s.clone() }
        }));
        eprintln!("║   thumbnail_base64: {} chars", item.thumbnail_base64.as_ref().map(|s| s.len()).unwrap_or(0));
        eprintln!("║   image_path: {:?}", item.image_path);
        eprintln!("║   source_app: {:?}", item.source_app);
        eprintln!("║   replaced_item_id: {:?}", replaced_item_id);

        match self.db.insert_item(&item) {
            Ok(()) => {
                eprintln!("║   ✓ DATABASE INSERT SUCCESS");
            }
            Err(e) => {
                eprintln!("║   ✗ DATABASE INSERT FAILED: {}", e);
                eprintln!("║   CRITICAL: Item {} is LOST!", item.id);
                eprintln!("╚═══════════════════════════════════════════════════════════");
                return;
            }
        }

        if let Ok(limit) = self.db.get_history_limit() {
            if let Err(e) = self.db.prune_oldest(limit) {
                eprintln!("║   Warning: prune_oldest failed: {}", e);
            }
        }

        eprintln!("║   Emitting clipboard-changed event...");
        let payload = ClipboardChangedPayload {
            item: item.clone(),
            replaced_item_id,
        };
        match self.app_handle.emit("clipboard-changed", &payload) {
            Ok(()) => {
                eprintln!("║   ✓ EVENT EMITTED SUCCESSFULLY");
            }
            Err(e) => {
                eprintln!("║   ✗ EVENT EMIT FAILED: {}", e);
                eprintln!("║   Item saved to DB but frontend not notified!");
            }
        }
        eprintln!("╚═══════════════════════════════════════════════════════════");
    }

    /// Try to get the source application name and icon
    #[cfg(target_os = "windows")]
    fn get_source_app_info(&self) -> (Option<String>, Option<String>) {
        get_clipboard_owner_app_info()
    }

    /// Try to get the source application name and icon (macOS)
    #[cfg(target_os = "macos")]
    fn get_source_app_info(&self) -> (Option<String>, Option<String>) {
        get_frontmost_app_info()
    }
}

/// Get the frontmost application name and icon on macOS using NSWorkspace
#[cfg(target_os = "macos")]
fn get_frontmost_app_info() -> (Option<String>, Option<String>) {
    use objc2_app_kit::{NSRunningApplication, NSWorkspace};

    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let frontmost_app: Option<objc2::rc::Retained<NSRunningApplication>> =
            workspace.frontmostApplication();

        if let Some(app) = frontmost_app {
            // Get app name
            let name = app
                .localizedName()
                .map(|n| n.to_string());

            // Get app icon
            let icon = get_app_icon_base64(&app);

            return (name, icon);
        }
        (None, None)
    }
}

/// Extract application icon as base64-encoded PNG (32x32)
#[cfg(target_os = "macos")]
fn get_app_icon_base64(app: &objc2_app_kit::NSRunningApplication) -> Option<String> {
    use base64::Engine;
    use objc2::rc::Retained;
    use objc2::ClassType;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSImage};
    use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize};

    unsafe {
        // Get the app's icon (NSImage)
        let icon: Option<Retained<NSImage>> = app.icon();
        let icon = icon?;

        // Target size: 32x32 for consistency
        let target_size = NSSize::new(32.0, 32.0);

        // Create a new NSImage with the target size
        let resized_image = NSImage::initWithSize(NSImage::alloc(), target_size);

        // Lock focus and draw the original icon scaled (deprecated but works)
        #[allow(deprecated)]
        resized_image.lockFocus();

        // Draw the original icon into the resized image
        let source_rect = NSRect::new(
            NSPoint::new(0.0, 0.0),
            icon.size(),
        );
        let dest_rect = NSRect::new(
            NSPoint::new(0.0, 0.0),
            target_size,
        );

        icon.drawInRect_fromRect_operation_fraction(
            dest_rect,
            source_rect,
            NSCompositingOperation::SourceOver,
            1.0,
        );

        #[allow(deprecated)]
        resized_image.unlockFocus();

        // Get TIFF representation of the resized image
        let tiff_data = resized_image.TIFFRepresentation()?;

        // Create bitmap image rep from TIFF data
        let bitmap_rep = NSBitmapImageRep::imageRepWithData(&tiff_data)?;

        // Convert to PNG
        let png_data = bitmap_rep
            .representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())?;

        // Convert NSData to bytes slice and encode as base64
        let bytes = png_data.bytes();
        let base64_str = base64::engine::general_purpose::STANDARD.encode(bytes);
        Some(base64_str)
    }
}

// ============================================================================
// Windows Source App Detection
// ============================================================================

/// Get the clipboard owner application name on Windows
#[cfg(target_os = "windows")]
fn get_clipboard_owner_app_info() -> (Option<String>, Option<String>) {
    use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
    use windows::Win32::System::DataExchange::GetClipboardOwner;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    unsafe {
        // Get the clipboard owner window handle
        let hwnd: HWND = GetClipboardOwner();
        if hwnd.0.is_null() {
            return (None, None);
        }

        // Get the process ID from the window handle
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return (None, None);
        }

        // Open the process to query its module name
        let process_handle: windows::core::Result<HANDLE> =
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id);

        let process_handle = match process_handle {
            Ok(handle) if !handle.is_invalid() => handle,
            _ => return (None, None),
        };

        // Get the executable path
        let mut exe_path_buffer = [0u16; 260]; // MAX_PATH
        let path_len = GetModuleFileNameExW(Some(process_handle), None, &mut exe_path_buffer);

        // Close the process handle
        let _ = CloseHandle(process_handle);

        if path_len == 0 {
            return (None, None);
        }

        // Convert UTF-16 to String
        let exe_path = String::from_utf16_lossy(&exe_path_buffer[..path_len as usize]);

        // Extract the executable name from the path
        let app_name = extract_app_name_from_path(&exe_path);

        // Extract the application icon as base64 PNG
        let icon_base64 = extract_app_icon_base64(&exe_path);

        (app_name, icon_base64)
    }
}

/// Extract a friendly application name from an executable path
#[cfg(target_os = "windows")]
fn extract_app_name_from_path(exe_path: &str) -> Option<String> {
    use std::path::Path;

    let path = Path::new(exe_path);
    let file_name = path.file_stem()?.to_str()?;

    // Convert to friendly name
    let friendly_name = match file_name.to_lowercase().as_str() {
        "chrome" => "Chrome",
        "firefox" => "Firefox",
        "msedge" => "Edge",
        "code" => "Visual Studio Code",
        "notepad" => "Notepad",
        "notepad++" => "Notepad++",
        "explorer" => "Explorer",
        "outlook" => "Outlook",
        "excel" => "Excel",
        "winword" => "Word",
        "powerpnt" => "PowerPoint",
        "teams" => "Teams",
        "slack" => "Slack",
        "discord" => "Discord",
        "spotify" => "Spotify",
        "terminal" => "Terminal",
        "windowsterminal" => "Windows Terminal",
        "powershell" => "PowerShell",
        "cmd" => "Command Prompt",
        _ => {
            // Capitalize first letter for unknown apps
            let mut chars = file_name.chars();
            match chars.next() {
                Some(first) => {
                    let capitalized: String =
                        first.to_uppercase().chain(chars).collect();
                    return Some(capitalized);
                }
                None => return None,
            }
        }
    };

    Some(friendly_name.to_string())
}

/// Extract application icon as base64-encoded PNG (32x32) from executable path
#[cfg(target_os = "windows")]
fn extract_app_icon_base64(exe_path: &str) -> Option<String> {
    use base64::Engine;
    use std::mem::MaybeUninit;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, DrawIconEx, GetDC, ReleaseDC, DI_NORMAL,
    };

    unsafe {
        // Convert path to wide string
        let wide_path: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();

        // Get the icon handle using SHGetFileInfoW
        let mut file_info: SHFILEINFOW = std::mem::zeroed();
        let result = SHGetFileInfoW(
            windows::core::PCWSTR::from_raw(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || file_info.hIcon.is_invalid() {
            return None;
        }

        let hicon = file_info.hIcon;

        // Target size: 32x32 (matching macOS implementation)
        let icon_size: i32 = 32;

        // Get device context for the screen
        let screen_dc = GetDC(HWND::default());
        if screen_dc.is_invalid() {
            DestroyIcon(hicon).ok();
            return None;
        }

        // Create a compatible DC for drawing
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        if mem_dc.is_invalid() {
            ReleaseDC(HWND::default(), screen_dc);
            DestroyIcon(hicon).ok();
            return None;
        }

        // Create a bitmap to draw into
        let bitmap = CreateCompatibleBitmap(screen_dc, icon_size, icon_size);
        if bitmap.is_invalid() {
            DeleteDC(mem_dc).ok();
            ReleaseDC(HWND::default(), screen_dc);
            DestroyIcon(hicon).ok();
            return None;
        }

        // Select the bitmap into the DC
        let old_bitmap = SelectObject(mem_dc, bitmap);

        // Draw the icon onto the bitmap
        let draw_result = DrawIconEx(
            mem_dc,
            0,
            0,
            hicon,
            icon_size,
            icon_size,
            0,
            None,
            DI_NORMAL,
        );

        if draw_result.is_err() {
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap).ok();
            DeleteDC(mem_dc).ok();
            ReleaseDC(HWND::default(), screen_dc);
            DestroyIcon(hicon).ok();
            return None;
        }

        // Prepare BITMAPINFO for extracting pixels
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = icon_size;
        bmi.bmiHeader.biHeight = -icon_size; // Negative for top-down DIB
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32; // RGBA
        bmi.bmiHeader.biCompression = BI_RGB.0;

        // Allocate buffer for pixel data (BGRA format)
        let buffer_size = (icon_size * icon_size * 4) as usize;
        let mut pixels: Vec<u8> = vec![0; buffer_size];

        // Get the bitmap bits
        let scanlines = GetDIBits(
            mem_dc,
            bitmap,
            0,
            icon_size as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Clean up GDI objects
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(bitmap).ok();
        DeleteDC(mem_dc).ok();
        ReleaseDC(HWND::default(), screen_dc);
        DestroyIcon(hicon).ok();

        if scanlines == 0 {
            return None;
        }

        // Convert BGRA to RGBA
        for i in (0..buffer_size).step_by(4) {
            pixels.swap(i, i + 2); // Swap B and R
        }

        // Create PNG using the image crate
        let img_buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
            icon_size as u32,
            icon_size as u32,
            pixels,
        )?;

        // Encode as PNG
        let mut png_data: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
        encoder
            .encode(
                img_buffer.as_raw(),
                icon_size as u32,
                icon_size as u32,
                image::ExtendedColorType::Rgba8,
            )
            .ok()?;

        // Encode as base64
        let base64_str = base64::engine::general_purpose::STANDARD.encode(&png_data);
        Some(base64_str)
    }
}

// ============================================================================
// Windows Implementation (uses clipboard-master)
// ============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use clipboard_master::{CallbackResult, ClipboardHandler, Master};

    impl ClipboardHandler for ClipboardMonitorHandler {
        fn on_clipboard_change(&mut self) -> CallbackResult {
            if SHOULD_STOP.load(Ordering::SeqCst) {
                return CallbackResult::Stop;
            }
            self.process_clipboard_change();
            CallbackResult::Next
        }

        fn on_clipboard_error(&mut self, error: std::io::Error) -> CallbackResult {
            eprintln!("Clipboard monitor error: {}", error);
            if SHOULD_STOP.load(Ordering::SeqCst) {
                return CallbackResult::Stop;
            }
            CallbackResult::Next
        }
    }

    pub fn start_monitoring_impl(
        app_handle: AppHandle,
        db: Arc<Database>,
    ) -> Result<JoinHandle<()>, String> {
        let handle = thread::spawn(move || {
            let handler = ClipboardMonitorHandler::new(app_handle, db);
            let mut master = Master::new(handler);

            if let Err(e) = master.run() {
                eprintln!("Clipboard monitor stopped with error: {}", e);
            }
        });

        Ok(handle)
    }
}

// ============================================================================
// macOS Implementation (polling-based with arboard)
// ============================================================================
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::time::Duration;

    pub fn start_monitoring_impl(
        app_handle: AppHandle,
        db: Arc<Database>,
    ) -> Result<JoinHandle<()>, String> {
        let handle = thread::spawn(move || {
            let handler = ClipboardMonitorHandler::new(app_handle, db);

            // Use pasteboard changeCount for reliable change detection
            // This increments every time the clipboard changes, even for same content
            let mut last_change_count = clipboard_reader::get_change_count();

            // Poll for clipboard changes using changeCount
            // 100ms polling for responsive UX
            while !SHOULD_STOP.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));

                let current_change_count = clipboard_reader::get_change_count();
                if current_change_count != last_change_count {
                    last_change_count = current_change_count;

                    // Small delay to let the source app finish writing to clipboard
                    // Some apps write to clipboard asynchronously
                    thread::sleep(Duration::from_millis(50));

                    // Check if pasteboard actually has content before processing
                    // This filters out clipboard clears and transient states
                    if clipboard_reader::pasteboard_has_content() {
                        handler.process_clipboard_change();
                    } else {
                        eprintln!("[ClipboardMonitor] changeCount {} but pasteboard is empty - skipping (likely a clear or transient state)", current_change_count);
                    }
                }
            }
        });

        Ok(handle)
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Start the clipboard monitor
pub fn start_monitoring(app_handle: AppHandle, db: Arc<Database>) -> Result<(), String> {
    let monitor_mutex = MONITOR_HANDLE.get_or_init(|| Mutex::new(None));

    let mut guard = monitor_mutex
        .lock()
        .map_err(|e| format!("Failed to lock monitor: {}", e))?;

    if guard.is_some() {
        return Ok(());
    }

    SHOULD_STOP.store(false, Ordering::SeqCst);

    let handle = platform::start_monitoring_impl(app_handle, db)?;
    *guard = Some(handle);

    Ok(())
}

/// Stop the clipboard monitor
pub fn stop_monitoring() {
    SHOULD_STOP.store(true, Ordering::SeqCst);

    if let Some(monitor_mutex) = MONITOR_HANDLE.get() {
        if let Ok(mut guard) = monitor_mutex.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.join();
            }
        }
    }
}

/// Check if monitor is running
pub fn is_monitoring() -> bool {
    if let Some(monitor_mutex) = MONITOR_HANDLE.get() {
        if let Ok(guard) = monitor_mutex.lock() {
            return guard.is_some() && !SHOULD_STOP.load(Ordering::SeqCst);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_content(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("Hello");
        let hash2 = hash_content("Hello");
        let hash3 = hash_content("World");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
