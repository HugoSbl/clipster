use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Content type for clipboard items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Image,
    Files,
    Link,
    Audio,
    Documents,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Files => "files",
            ContentType::Link => "link",
            ContentType::Audio => "audio",
            ContentType::Documents => "documents",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "image" => Some(ContentType::Image),
            "files" => Some(ContentType::Files),
            "link" => Some(ContentType::Link),
            "audio" => Some(ContentType::Audio),
            "documents" => Some(ContentType::Documents),
            _ => None,
        }
    }

    /// Check if text content looks like a URL
    pub fn detect_from_text(text: &str) -> Self {
        let trimmed = text.trim();

        // Check if it's a URL
        if Self::is_url(trimmed) {
            return ContentType::Link;
        }

        ContentType::Text
    }

    /// Check if text is a URL
    fn is_url(text: &str) -> bool {
        let lower = text.to_lowercase();
        // Must start with a protocol or www
        if lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("ftp://")
            || lower.starts_with("file://")
            || lower.starts_with("www.")
        {
            // Basic validation: no newlines and contains a dot
            !text.contains('\n') && text.contains('.')
        } else {
            false
        }
    }

    /// Check if file paths contain audio or document files
    pub fn detect_from_files(paths: &[String]) -> Self {
        let audio_extensions = [
            "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "aiff", "alac", "opus"
        ];

        let document_extensions = [
            // PDF
            "pdf",
            // Microsoft Office
            "doc", "docx", "xls", "xlsx", "ppt", "pptx",
            // OpenDocument
            "odt", "ods", "odp",
            // Apple iWork
            "pages", "numbers", "keynote",
            // Other common document formats
            "rtf", "txt", "csv",
        ];

        let all_audio = !paths.is_empty() && paths.iter().all(|path| {
            let lower = path.to_lowercase();
            audio_extensions.iter().any(|ext| lower.ends_with(&format!(".{}", ext)))
        });

        let all_documents = !paths.is_empty() && paths.iter().all(|path| {
            let lower = path.to_lowercase();
            document_extensions.iter().any(|ext| lower.ends_with(&format!(".{}", ext)))
        });

        if all_audio {
            ContentType::Audio
        } else if all_documents {
            ContentType::Documents
        } else {
            ContentType::Files
        }
    }
}

impl ToSql for ContentType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for ContentType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| {
            ContentType::from_str(s).ok_or_else(|| FromSqlError::InvalidType)
        })
    }
}

/// Represents a clipboard history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier (UUID v4)
    pub id: String,

    /// Type of content stored
    pub content_type: ContentType,

    /// Text content (for text items) or file paths JSON (for files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_text: Option<String>,

    /// Thumbnail as base64-encoded PNG (for images)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_base64: Option<String>,

    /// Original image data path (for images, stored externally)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,

    /// Source application name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_app: Option<String>,

    /// Source application icon as base64-encoded PNG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_app_icon: Option<String>,

    /// Timestamp when item was captured
    pub created_at: DateTime<Utc>,

    /// Associated pinboard ID (if pinned)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinboard_id: Option<String>,

    /// Whether this item is favorited/starred
    #[serde(default)]
    pub is_favorite: bool,
}

impl ClipboardItem {
    /// Create a new text clipboard item (auto-detects if it's a URL)
    pub fn new_text(text: String, source_app: Option<String>, source_app_icon: Option<String>) -> Self {
        let content_type = ContentType::detect_from_text(&text);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type,
            content_text: Some(text),
            thumbnail_base64: None,
            image_path: None,
            source_app,
            source_app_icon,
            created_at: Utc::now(),
            pinboard_id: None,
            is_favorite: false,
        }
    }

    /// Create a new link clipboard item
    pub fn new_link(url: String, source_app: Option<String>, source_app_icon: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Link,
            content_text: Some(url),
            thumbnail_base64: None,
            image_path: None,
            source_app,
            source_app_icon,
            created_at: Utc::now(),
            pinboard_id: None,
            is_favorite: false,
        }
    }

    /// Create a new image clipboard item
    pub fn new_image(
        thumbnail_base64: Option<String>,
        image_path: String,
        source_app: Option<String>,
        source_app_icon: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Image,
            content_text: None,
            thumbnail_base64,
            image_path: Some(image_path),
            source_app,
            source_app_icon,
            created_at: Utc::now(),
            pinboard_id: None,
            is_favorite: false,
        }
    }

    /// Create a new files clipboard item (auto-detects if all files are audio)
    pub fn new_files(file_paths: Vec<String>, source_app: Option<String>, source_app_icon: Option<String>) -> Self {
        Self::new_files_with_thumbnail(file_paths, source_app, source_app_icon, None)
    }

    /// Create a new files clipboard item with optional thumbnail
    pub fn new_files_with_thumbnail(
        file_paths: Vec<String>,
        source_app: Option<String>,
        source_app_icon: Option<String>,
        thumbnail_base64: Option<String>,
    ) -> Self {
        let content_type = ContentType::detect_from_files(&file_paths);
        let paths_json = serde_json::to_string(&file_paths).unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type,
            content_text: Some(paths_json),
            thumbnail_base64,
            image_path: None,
            source_app,
            source_app_icon,
            created_at: Utc::now(),
            pinboard_id: None,
            is_favorite: false,
        }
    }

    /// Create a new audio files clipboard item
    pub fn new_audio(file_paths: Vec<String>, source_app: Option<String>, source_app_icon: Option<String>) -> Self {
        let paths_json = serde_json::to_string(&file_paths).unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Audio,
            content_text: Some(paths_json),
            thumbnail_base64: None,
            image_path: None,
            source_app,
            source_app_icon,
            created_at: Utc::now(),
            pinboard_id: None,
            is_favorite: false,
        }
    }

    /// Get file paths for Files, Audio, or Documents type items
    pub fn get_file_paths(&self) -> Option<Vec<String>> {
        if !matches!(self.content_type, ContentType::Files | ContentType::Audio | ContentType::Documents) {
            return None;
        }
        self.content_text
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
    }

    /// Create from a rusqlite Row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let created_at_str: String = row.get("created_at")?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Self {
            id: row.get("id")?,
            content_type: row.get("content_type")?,
            content_text: row.get("content_text")?,
            thumbnail_base64: row.get("thumbnail_base64")?,
            image_path: row.get("image_path")?,
            source_app: row.get("source_app")?,
            source_app_icon: row.get("source_app_icon")?,
            created_at,
            pinboard_id: row.get("pinboard_id")?,
            is_favorite: row.get::<_, i32>("is_favorite")? != 0,
        })
    }

    /// Get a preview string for display (truncated text or description)
    pub fn preview(&self, max_len: usize) -> String {
        match self.content_type {
            ContentType::Text => {
                let text = self.content_text.as_deref().unwrap_or("");
                if text.len() > max_len {
                    format!("{}...", &text[..max_len])
                } else {
                    text.to_string()
                }
            }
            ContentType::Link => {
                let url = self.content_text.as_deref().unwrap_or("");
                // Extract domain for preview
                if let Some(start) = url.find("://") {
                    let after_proto = &url[start + 3..];
                    if let Some(end) = after_proto.find('/') {
                        after_proto[..end].to_string()
                    } else {
                        after_proto.to_string()
                    }
                } else if url.starts_with("www.") {
                    if let Some(end) = url[4..].find('/') {
                        url[..4 + end].to_string()
                    } else {
                        url.to_string()
                    }
                } else {
                    url.to_string()
                }
            }
            ContentType::Image => "[Image]".to_string(),
            ContentType::Files => {
                if let Some(paths) = self.get_file_paths() {
                    if paths.len() == 1 {
                        paths[0].clone()
                    } else {
                        format!("{} files", paths.len())
                    }
                } else {
                    "[Files]".to_string()
                }
            }
            ContentType::Audio => {
                if let Some(paths) = self.get_file_paths() {
                    if paths.len() == 1 {
                        // Get just the filename
                        paths[0]
                            .rsplit(['/', '\\'])
                            .next()
                            .unwrap_or(&paths[0])
                            .to_string()
                    } else {
                        format!("{} audio files", paths.len())
                    }
                } else {
                    "[Audio]".to_string()
                }
            }
            ContentType::Documents => {
                if let Some(paths) = self.get_file_paths() {
                    if paths.len() == 1 {
                        // Get just the filename
                        paths[0]
                            .rsplit(['/', '\\'])
                            .next()
                            .unwrap_or(&paths[0])
                            .to_string()
                    } else {
                        format!("{} documents", paths.len())
                    }
                } else {
                    "[Documents]".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_serialization() {
        assert_eq!(ContentType::Text.as_str(), "text");
        assert_eq!(ContentType::Image.as_str(), "image");
        assert_eq!(ContentType::Files.as_str(), "files");
        assert_eq!(ContentType::Link.as_str(), "link");
        assert_eq!(ContentType::Audio.as_str(), "audio");
    }

    #[test]
    fn test_new_text_item() {
        let item = ClipboardItem::new_text("Hello".to_string(), Some("Notepad".to_string()), None);
        assert_eq!(item.content_type, ContentType::Text);
        assert_eq!(item.content_text, Some("Hello".to_string()));
        assert_eq!(item.source_app, Some("Notepad".to_string()));
    }

    #[test]
    fn test_url_detection() {
        let item = ClipboardItem::new_text("https://github.com/rust-lang".to_string(), None, None);
        assert_eq!(item.content_type, ContentType::Link);

        let item = ClipboardItem::new_text("http://example.com".to_string(), None, None);
        assert_eq!(item.content_type, ContentType::Link);

        let item = ClipboardItem::new_text("www.google.com".to_string(), None, None);
        assert_eq!(item.content_type, ContentType::Link);

        // Plain text should stay as Text
        let item = ClipboardItem::new_text("Hello world".to_string(), None, None);
        assert_eq!(item.content_type, ContentType::Text);

        // Multiline should not be detected as URL
        let item = ClipboardItem::new_text("https://example.com\nmore text".to_string(), None, None);
        assert_eq!(item.content_type, ContentType::Text);
    }

    #[test]
    fn test_audio_detection() {
        let paths = vec!["/music/song.mp3".to_string()];
        let item = ClipboardItem::new_files(paths, None, None);
        assert_eq!(item.content_type, ContentType::Audio);

        let paths = vec!["/music/track1.wav".to_string(), "/music/track2.flac".to_string()];
        let item = ClipboardItem::new_files(paths, None, None);
        assert_eq!(item.content_type, ContentType::Audio);

        // Mixed files should be Files, not Audio
        let paths = vec!["/doc.pdf".to_string(), "/song.mp3".to_string()];
        let item = ClipboardItem::new_files(paths, None, None);
        assert_eq!(item.content_type, ContentType::Files);
    }

    #[test]
    fn test_file_paths() {
        let paths = vec!["C:\\file1.txt".to_string(), "C:\\file2.txt".to_string()];
        let item = ClipboardItem::new_files(paths.clone(), None, None);
        assert_eq!(item.get_file_paths(), Some(paths));
    }
}
