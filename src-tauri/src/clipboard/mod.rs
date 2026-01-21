// Cross-platform clipboard module
// Provides unified API for clipboard operations on Windows and macOS

pub mod clipboard_monitor;
pub mod clipboard_reader;

// Re-export common types
pub use clipboard_reader::{ClipboardContent, ImageData};
