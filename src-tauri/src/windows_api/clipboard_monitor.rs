use crate::models::{ClipboardItem, ContentType};
use crate::storage::{file_storage, Database, FileStorage};
use crate::windows_api::clipboard_reader::{self, ClipboardContent};
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter};

/// Global monitor instance
static MONITOR_HANDLE: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();
static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
static LAST_CONTENT_HASH: AtomicU64 = AtomicU64::new(0);

/// Event payload for clipboard changes
#[derive(Clone, serde::Serialize)]
pub struct ClipboardChangedPayload {
    pub item: ClipboardItem,
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

    /// Calculate hash of content for deduplication
    fn hash_content(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate hash of bytes for image deduplication
    fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Process new clipboard content
    fn process_clipboard_change(&self) {
        // Read clipboard content based on detected format
        let content = clipboard_reader::read_clipboard();

        match content {
            ClipboardContent::Text(text) => self.process_text(text),
            ClipboardContent::Image(image_data) => self.process_image(image_data),
            ClipboardContent::Files(files) => self.process_files(files),
            ClipboardContent::Empty => {} // Skip empty content
        }
    }

    /// Process text clipboard content
    fn process_text(&self, text: String) {
        if text.trim().is_empty() {
            return;
        }

        // Check for duplicate content (prevents infinite loops)
        let content_hash = Self::hash_content(&text);
        let last_hash = LAST_CONTENT_HASH.load(Ordering::SeqCst);
        if content_hash == last_hash {
            return;
        }
        LAST_CONTENT_HASH.store(content_hash, Ordering::SeqCst);

        // Check if content already exists in database
        if let Ok(true) = self.db.content_exists(&text) {
            return;
        }

        // Create and save the clipboard item
        let item = ClipboardItem::new_text(text, self.get_source_app());
        self.save_and_emit(item);
    }

    /// Process image clipboard content
    fn process_image(&self, image_data: clipboard_reader::ImageData) {
        // Check for duplicate image (using raw DIB data hash)
        let content_hash = Self::hash_bytes(&image_data.dib_data);
        let last_hash = LAST_CONTENT_HASH.load(Ordering::SeqCst);
        if content_hash == last_hash {
            return;
        }
        LAST_CONTENT_HASH.store(content_hash, Ordering::SeqCst);

        // Generate ID for the image
        let id = uuid::Uuid::new_v4().to_string();

        // Decode DIB data and process
        match file_storage::decode_dib(&image_data.dib_data) {
            Ok(image) => {
                // Generate thumbnail
                let thumbnail_base64 = match file_storage::generate_thumbnail_default(&image) {
                    Ok(png_bytes) => file_storage::thumbnail_to_base64(&png_bytes),
                    Err(e) => {
                        eprintln!("Failed to generate thumbnail: {}", e);
                        return;
                    }
                };

                // Save full image to disk
                let image_path = match self.file_storage.save_image(&id, &image) {
                    Ok(path) => path.to_string_lossy().to_string(),
                    Err(e) => {
                        eprintln!("Failed to save image: {}", e);
                        return;
                    }
                };

                // Create clipboard item
                let item = ClipboardItem::new_image(thumbnail_base64, image_path, self.get_source_app());
                self.save_and_emit(item);
            }
            Err(e) => {
                eprintln!("Failed to decode DIB image: {}", e);
            }
        }
    }

    /// Process file list clipboard content
    fn process_files(&self, files: Vec<String>) {
        if files.is_empty() {
            return;
        }

        // Create hash from file paths
        let paths_str = files.join("|");
        let content_hash = Self::hash_content(&paths_str);
        let last_hash = LAST_CONTENT_HASH.load(Ordering::SeqCst);
        if content_hash == last_hash {
            return;
        }
        LAST_CONTENT_HASH.store(content_hash, Ordering::SeqCst);

        // Create clipboard item
        let item = ClipboardItem::new_files(files, self.get_source_app());
        self.save_and_emit(item);
    }

    /// Save item to database and emit event to frontend
    fn save_and_emit(&self, item: ClipboardItem) {
        if let Err(e) = self.db.insert_item(&item) {
            eprintln!("Failed to save clipboard item: {}", e);
            return;
        }

        // Prune old items if needed
        if let Ok(limit) = self.db.get_history_limit() {
            let _ = self.db.prune_oldest(limit);
        }

        // Emit event to frontend
        let payload = ClipboardChangedPayload { item };
        if let Err(e) = self.app_handle.emit("clipboard-changed", &payload) {
            eprintln!("Failed to emit clipboard-changed event: {}", e);
        }
    }

    /// Try to get the source application name (placeholder for now)
    fn get_source_app(&self) -> Option<String> {
        // TODO: Implement getting foreground window title
        // For now, return None
        None
    }
}

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

/// Start the clipboard monitor
/// Should be called once during app initialization
pub fn start_monitoring(app_handle: AppHandle, db: Arc<Database>) -> Result<(), String> {
    let monitor_mutex = MONITOR_HANDLE.get_or_init(|| Mutex::new(None));

    let mut guard = monitor_mutex
        .lock()
        .map_err(|e| format!("Failed to lock monitor: {}", e))?;

    // Already running
    if guard.is_some() {
        return Ok(());
    }

    SHOULD_STOP.store(false, Ordering::SeqCst);

    let handle = thread::spawn(move || {
        let handler = ClipboardMonitorHandler::new(app_handle, db);
        let mut master = Master::new(handler);

        // Run the clipboard monitoring loop
        if let Err(e) = master.run() {
            eprintln!("Clipboard monitor stopped with error: {}", e);
        }
    });

    *guard = Some(handle);
    Ok(())
}

/// Stop the clipboard monitor
pub fn stop_monitoring() {
    SHOULD_STOP.store(true, Ordering::SeqCst);

    if let Some(monitor_mutex) = MONITOR_HANDLE.get() {
        if let Ok(mut guard) = monitor_mutex.lock() {
            if let Some(handle) = guard.take() {
                // Give the thread a moment to stop gracefully
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
    use super::*;

    #[test]
    fn test_hash_content() {
        let hash1 = ClipboardMonitorHandler::hash_content("Hello");
        let hash2 = ClipboardMonitorHandler::hash_content("Hello");
        let hash3 = ClipboardMonitorHandler::hash_content("World");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
