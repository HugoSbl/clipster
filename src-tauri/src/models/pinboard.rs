use chrono::{DateTime, Utc};
use rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Represents a pinboard for organizing clipboard items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pinboard {
    /// Unique identifier (UUID v4)
    pub id: String,

    /// Display name
    pub name: String,

    /// Icon identifier (emoji or icon name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Sort order position
    pub position: i32,

    /// Timestamp when pinboard was created
    pub created_at: DateTime<Utc>,
}

impl Pinboard {
    /// Create a new pinboard
    pub fn new(name: String, icon: Option<String>, position: i32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            icon,
            position,
            created_at: Utc::now(),
        }
    }

    /// Create a default "Favorites" pinboard
    pub fn default_favorites() -> Self {
        Self::new("Favorites".to_string(), Some("star".to_string()), 0)
    }

    /// Create from a rusqlite Row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let created_at_str: String = row.get("created_at")?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            icon: row.get("icon")?,
            position: row.get("position")?,
            created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pinboard() {
        let pinboard = Pinboard::new("Work".to_string(), Some("briefcase".to_string()), 1);
        assert_eq!(pinboard.name, "Work");
        assert_eq!(pinboard.icon, Some("briefcase".to_string()));
        assert_eq!(pinboard.position, 1);
    }

    #[test]
    fn test_default_favorites() {
        let favorites = Pinboard::default_favorites();
        assert_eq!(favorites.name, "Favorites");
        assert_eq!(favorites.position, 0);
    }
}
