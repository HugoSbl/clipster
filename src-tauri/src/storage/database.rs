use crate::models::{ClipboardItem, ContentType, Pinboard};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;

/// Database wrapper for thread-safe SQLite operations
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Create a new database connection
    /// Uses app data directory: ~/.clipster/clipster.db
    pub fn new() -> Result<Self, String> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.run_migrations()?;

        Ok(db)
    }

    /// Create an in-memory database (for testing)
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory database: {}", e))?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.run_migrations()?;

        Ok(db)
    }

    /// Get the database file path
    fn get_db_path() -> Result<PathBuf, String> {
        let home = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| "Could not determine home directory".to_string())?;

        Ok(home.join(".clipster").join("clipster.db"))
    }

    /// Run database migrations
    fn run_migrations(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        // Create clipboard_items table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id TEXT PRIMARY KEY,
                content_type TEXT NOT NULL,
                content_text TEXT,
                thumbnail_base64 TEXT,
                image_path TEXT,
                source_app TEXT,
                source_app_icon TEXT,
                created_at TEXT NOT NULL,
                pinboard_id TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (pinboard_id) REFERENCES pinboards(id) ON DELETE SET NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create clipboard_items table: {}", e))?;

        // Migration: Add source_app_icon column if it doesn't exist
        let _ = conn.execute(
            "ALTER TABLE clipboard_items ADD COLUMN source_app_icon TEXT",
            [],
        );

        // Create pinboards table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pinboards (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon TEXT,
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create pinboards table: {}", e))?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create settings table: {}", e))?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_items_created_at
             ON clipboard_items(created_at DESC)",
            [],
        )
        .map_err(|e| format!("Failed to create created_at index: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_items_content_type
             ON clipboard_items(content_type)",
            [],
        )
        .map_err(|e| format!("Failed to create content_type index: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_items_pinboard
             ON clipboard_items(pinboard_id)",
            [],
        )
        .map_err(|e| format!("Failed to create pinboard_id index: {}", e))?;

        // Insert default settings if not present
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('history_limit', '500')",
            [],
        )
        .map_err(|e| format!("Failed to insert default settings: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('shortcut', 'Ctrl+Shift+V')",
            [],
        )
        .map_err(|e| format!("Failed to insert default shortcut: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('start_hidden', 'false')",
            [],
        )
        .map_err(|e| format!("Failed to insert default start_hidden: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'dark')",
            [],
        )
        .map_err(|e| format!("Failed to insert default theme: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('show_menu_bar_icon', 'true')",
            [],
        )
        .map_err(|e| format!("Failed to insert default show_menu_bar_icon: {}", e))?;

        Ok(())
    }

    // ==================== CLIPBOARD ITEMS ====================

    /// Insert a new clipboard item
    pub fn insert_item(&self, item: &ClipboardItem) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            "INSERT INTO clipboard_items
             (id, content_type, content_text, thumbnail_base64, image_path, source_app, source_app_icon, created_at, pinboard_id, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                item.id,
                item.content_type,
                item.content_text,
                item.thumbnail_base64,
                item.image_path,
                item.source_app,
                item.source_app_icon,
                item.created_at.to_rfc3339(),
                item.pinboard_id,
                item.is_favorite as i32,
            ],
        )
        .map_err(|e| format!("Failed to insert clipboard item: {}", e))?;

        Ok(())
    }

    /// Get clipboard history items with pagination
    /// Returns only items NOT in a pinboard, ordered by created_at DESC (newest first)
    pub fn get_items(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardItem>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, thumbnail_base64, image_path,
                        source_app, source_app_icon, created_at, pinboard_id, is_favorite
                 FROM clipboard_items
                 WHERE pinboard_id IS NULL
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let items = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                ClipboardItem::from_row(row)
            })
            .map_err(|e| format!("Failed to query items: {}", e))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect items: {}", e))?;

        Ok(items)
    }

    /// Get a single clipboard item by ID
    pub fn get_item(&self, id: &str) -> Result<Option<ClipboardItem>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, thumbnail_base64, image_path,
                        source_app, source_app_icon, created_at, pinboard_id, is_favorite
                 FROM clipboard_items
                 WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt
            .query_map(params![id], |row| ClipboardItem::from_row(row))
            .map_err(|e| format!("Failed to query item: {}", e))?;

        match rows.next() {
            Some(Ok(item)) => Ok(Some(item)),
            Some(Err(e)) => Err(format!("Failed to read item: {}", e)),
            None => Ok(None),
        }
    }

    /// Delete a clipboard item by ID
    pub fn delete_item(&self, id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let rows_affected = conn
            .execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete item: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Search clipboard items by text content
    pub fn search_items(&self, query: &str, limit: usize) -> Result<Vec<ClipboardItem>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let search_pattern = format!("%{}%", query);

        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, thumbnail_base64, image_path,
                        source_app, source_app_icon, created_at, pinboard_id, is_favorite
                 FROM clipboard_items
                 WHERE content_text LIKE ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        let items = stmt
            .query_map(params![search_pattern, limit as i64], |row| {
                ClipboardItem::from_row(row)
            })
            .map_err(|e| format!("Failed to search items: {}", e))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect search results: {}", e))?;

        Ok(items)
    }

    /// Get items by content type
    pub fn get_items_by_type(
        &self,
        content_type: ContentType,
        limit: usize,
    ) -> Result<Vec<ClipboardItem>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, thumbnail_base64, image_path,
                        source_app, source_app_icon, created_at, pinboard_id, is_favorite
                 FROM clipboard_items
                 WHERE content_type = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let items = stmt
            .query_map(params![content_type, limit as i64], |row| {
                ClipboardItem::from_row(row)
            })
            .map_err(|e| format!("Failed to query items by type: {}", e))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect items: {}", e))?;

        Ok(items)
    }

    /// Count history items (unpinned only)
    /// Pinboard items are saved permanently and not counted in history limit
    pub fn count_items(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clipboard_items WHERE pinboard_id IS NULL",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count items: {}", e))?;

        Ok(count as usize)
    }

    /// Prune oldest items to maintain history limit
    /// Keeps favorited items and items in pinboards
    pub fn prune_oldest(&self, keep_count: usize) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        // Delete oldest non-favorited, non-pinned items beyond the limit
        let deleted = conn
            .execute(
                "DELETE FROM clipboard_items
                 WHERE id IN (
                     SELECT id FROM clipboard_items
                     WHERE is_favorite = 0 AND pinboard_id IS NULL
                     ORDER BY created_at DESC
                     LIMIT -1 OFFSET ?1
                 )",
                params![keep_count as i64],
            )
            .map_err(|e| format!("Failed to prune items: {}", e))?;

        Ok(deleted)
    }

    /// Update item's pinboard assignment
    pub fn update_item_pinboard(
        &self,
        item_id: &str,
        pinboard_id: Option<&str>,
    ) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let rows_affected = conn
            .execute(
                "UPDATE clipboard_items SET pinboard_id = ?1 WHERE id = ?2",
                params![pinboard_id, item_id],
            )
            .map_err(|e| format!("Failed to update item pinboard: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Toggle item's favorite status
    pub fn toggle_item_favorite(&self, item_id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let rows_affected = conn
            .execute(
                "UPDATE clipboard_items SET is_favorite = NOT is_favorite WHERE id = ?1",
                params![item_id],
            )
            .map_err(|e| format!("Failed to toggle favorite: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Clear all non-favorited, non-pinned clipboard items
    pub fn clear_history(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let deleted = conn
            .execute(
                "DELETE FROM clipboard_items WHERE is_favorite = 0 AND pinboard_id IS NULL",
                [],
            )
            .map_err(|e| format!("Failed to clear history: {}", e))?;

        Ok(deleted)
    }

    /// Check if content already exists in UNPINNED history (not in pinboards)
    /// This allows the same content to exist both in history and in pinboards
    pub fn content_exists(&self, content_text: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM clipboard_items WHERE content_text = ?1 AND pinboard_id IS NULL LIMIT 1)",
                params![content_text],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check content existence: {}", e))?;

        Ok(exists)
    }

    /// Delete unpinned items with matching content (for "move to top" behavior)
    /// Returns the ID, source_app and source_app_icon of the deleted item (if any)
    /// Does NOT delete pinned items - they are preserved separately
    pub fn delete_unpinned_by_content(
        &self,
        content_text: &str,
    ) -> Result<Option<(String, Option<String>, Option<String>)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        // First, get the ID and source app info of the item we're about to delete
        let existing: Option<(String, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT id, source_app, source_app_icon FROM clipboard_items WHERE content_text = ?1 AND pinboard_id IS NULL LIMIT 1",
                params![content_text],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        if existing.is_some() {
            conn.execute(
                "DELETE FROM clipboard_items WHERE content_text = ?1 AND pinboard_id IS NULL",
                params![content_text],
            )
            .map_err(|e| format!("Failed to delete by content: {}", e))?;
        }

        Ok(existing)
    }

    // ==================== PINBOARDS ====================

    /// Insert a new pinboard
    pub fn insert_pinboard(&self, pinboard: &Pinboard) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            "INSERT INTO pinboards (id, name, icon, position, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                pinboard.id,
                pinboard.name,
                pinboard.icon,
                pinboard.position,
                pinboard.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| format!("Failed to insert pinboard: {}", e))?;

        Ok(())
    }

    /// Get all pinboards ordered by position
    pub fn get_pinboards(&self) -> Result<Vec<Pinboard>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, icon, position, created_at
                 FROM pinboards
                 ORDER BY position ASC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let pinboards = stmt
            .query_map([], |row| Pinboard::from_row(row))
            .map_err(|e| format!("Failed to query pinboards: {}", e))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect pinboards: {}", e))?;

        Ok(pinboards)
    }

    /// Get items in a specific pinboard
    pub fn get_pinboard_items(
        &self,
        pinboard_id: &str,
        limit: usize,
    ) -> Result<Vec<ClipboardItem>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, content_type, content_text, thumbnail_base64, image_path,
                        source_app, source_app_icon, created_at, pinboard_id, is_favorite
                 FROM clipboard_items
                 WHERE pinboard_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let items = stmt
            .query_map(params![pinboard_id, limit as i64], |row| {
                ClipboardItem::from_row(row)
            })
            .map_err(|e| format!("Failed to query pinboard items: {}", e))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect items: {}", e))?;

        Ok(items)
    }

    /// Update pinboard details
    pub fn update_pinboard(
        &self,
        id: &str,
        name: &str,
        icon: Option<&str>,
    ) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let rows_affected = conn
            .execute(
                "UPDATE pinboards SET name = ?1, icon = ?2 WHERE id = ?3",
                params![name, icon, id],
            )
            .map_err(|e| format!("Failed to update pinboard: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Delete a pinboard (items will have pinboard_id set to NULL)
    pub fn delete_pinboard(&self, id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let rows_affected = conn
            .execute("DELETE FROM pinboards WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete pinboard: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Get a single pinboard by ID
    pub fn get_pinboard(&self, id: &str) -> Result<Option<Pinboard>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, icon, position, created_at
                 FROM pinboards
                 WHERE id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt
            .query_map(params![id], |row| Pinboard::from_row(row))
            .map_err(|e| format!("Failed to query pinboard: {}", e))?;

        match rows.next() {
            Some(Ok(pinboard)) => Ok(Some(pinboard)),
            Some(Err(e)) => Err(format!("Failed to read pinboard: {}", e)),
            None => Ok(None),
        }
    }

    /// Reorder pinboards by updating their positions
    /// Takes a list of pinboard IDs in the desired order
    pub fn reorder_pinboards(&self, pinboard_ids: &[String]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        for (position, id) in pinboard_ids.iter().enumerate() {
            conn.execute(
                "UPDATE pinboards SET position = ?1 WHERE id = ?2",
                params![position as i32, id],
            )
            .map_err(|e| format!("Failed to update pinboard position: {}", e))?;
        }

        Ok(())
    }

    // ==================== SETTINGS ====================

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let result: SqliteResult<String> =
            conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| {
                row.get(0)
            });

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get setting: {}", e)),
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("Failed to set setting: {}", e))?;

        Ok(())
    }

    /// Get history limit setting
    pub fn get_history_limit(&self) -> Result<usize, String> {
        let limit_str = self.get_setting("history_limit")?.unwrap_or_else(|| "500".to_string());
        limit_str
            .parse()
            .map_err(|_| "Invalid history_limit value".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new_in_memory().expect("Failed to create in-memory database");
        assert!(db.count_items().unwrap() == 0);
    }

    #[test]
    fn test_insert_and_get_item() {
        let db = Database::new_in_memory().unwrap();

        let item = ClipboardItem::new_text("Hello, World!".to_string(), Some("Test".to_string()), None);
        db.insert_item(&item).unwrap();

        let items = db.get_items(10, 0).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content_text, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_delete_item() {
        let db = Database::new_in_memory().unwrap();

        let item = ClipboardItem::new_text("To delete".to_string(), None, None);
        let id = item.id.clone();
        db.insert_item(&item).unwrap();

        assert_eq!(db.count_items().unwrap(), 1);

        db.delete_item(&id).unwrap();
        assert_eq!(db.count_items().unwrap(), 0);
    }

    #[test]
    fn test_search_items() {
        let db = Database::new_in_memory().unwrap();

        db.insert_item(&ClipboardItem::new_text("Hello World".to_string(), None, None))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("Goodbye World".to_string(), None, None))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("Hello Rust".to_string(), None, None))
            .unwrap();

        let results = db.search_items("Hello", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search_items("World", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search_items("Rust", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_prune_oldest() {
        let db = Database::new_in_memory().unwrap();

        // Insert 10 items
        for i in 0..10 {
            let item = ClipboardItem::new_text(format!("Item {}", i), None, None);
            db.insert_item(&item).unwrap();
        }

        assert_eq!(db.count_items().unwrap(), 10);

        // Prune to keep only 5
        db.prune_oldest(5).unwrap();
        assert_eq!(db.count_items().unwrap(), 5);
    }

    #[test]
    fn test_prune_preserves_pinned_items() {
        let db = Database::new_in_memory().unwrap();

        // Create a pinboard
        let pinboard = Pinboard::new("Test".to_string(), None, 0);
        let pinboard_id = pinboard.id.clone();
        db.insert_pinboard(&pinboard).unwrap();

        // Insert 10 history items
        for i in 0..10 {
            let item = ClipboardItem::new_text(format!("History {}", i), None, None);
            db.insert_item(&item).unwrap();
        }

        // Insert 5 pinned items
        for i in 0..5 {
            let item = ClipboardItem::new_text(format!("Pinned {}", i), None, None);
            let item_id = item.id.clone();
            db.insert_item(&item).unwrap();
            db.update_item_pinboard(&item_id, Some(&pinboard_id)).unwrap();
        }

        // History count should be 10 (pinned items not counted)
        assert_eq!(db.count_items().unwrap(), 10);

        // Prune to keep only 3 history items
        db.prune_oldest(3).unwrap();

        // History count should now be 3
        assert_eq!(db.count_items().unwrap(), 3);

        // But pinned items should still exist
        let pinboard_items = db.get_pinboard_items(&pinboard_id, 100).unwrap();
        assert_eq!(pinboard_items.len(), 5);
    }

    #[test]
    fn test_settings() {
        let db = Database::new_in_memory().unwrap();

        // Default value
        assert_eq!(db.get_setting("history_limit").unwrap(), Some("500".to_string()));

        // Update value
        db.set_setting("history_limit", "1000").unwrap();
        assert_eq!(db.get_setting("history_limit").unwrap(), Some("1000".to_string()));

        // New key
        db.set_setting("custom_key", "custom_value").unwrap();
        assert_eq!(db.get_setting("custom_key").unwrap(), Some("custom_value".to_string()));
    }

    #[test]
    fn test_pinboards() {
        let db = Database::new_in_memory().unwrap();

        let pinboard = Pinboard::new("Work".to_string(), Some("briefcase".to_string()), 0);
        let pinboard_id = pinboard.id.clone();
        db.insert_pinboard(&pinboard).unwrap();

        let pinboards = db.get_pinboards().unwrap();
        assert_eq!(pinboards.len(), 1);
        assert_eq!(pinboards[0].name, "Work");

        // Add item to pinboard
        let item = ClipboardItem::new_text("Work item".to_string(), None, None);
        let item_id = item.id.clone();
        db.insert_item(&item).unwrap();
        db.update_item_pinboard(&item_id, Some(&pinboard_id)).unwrap();

        let pinboard_items = db.get_pinboard_items(&pinboard_id, 10).unwrap();
        assert_eq!(pinboard_items.len(), 1);
    }

    #[test]
    fn test_content_deduplication() {
        let db = Database::new_in_memory().unwrap();

        let item = ClipboardItem::new_text("Duplicate content".to_string(), None, None);
        db.insert_item(&item).unwrap();

        assert!(db.content_exists("Duplicate content").unwrap());
        assert!(!db.content_exists("Non-existent content").unwrap());
    }
}
