use rusqlite::Row;
use serde::Serialize;

/// Format: YYYY-MM-DD
pub type DbDay = String;
/// Format: 0xRRGGBB (ex. `0xFF00FF`).
pub type DbColor = u32;

/// Unix timestamp, in seconds.
pub type DbTimestamp = i64;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DbTodo {
    /// UUIDv7.
    pub id: String,
    pub day: DbDay,
    pub title: String,
    pub description: Option<String>,
    pub completed: i8,
    /// Fractional index key.
    pub position: String,
    pub created_at: DbTimestamp,
    pub updated_at: DbTimestamp,
    pub completed_at: Option<DbTimestamp>,
    pub deleted_at: Option<DbTimestamp>,
}

impl DbTodo {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            day: row.get("day")?,
            title: row.get("title")?,
            description: row.get("description")?,
            completed: row.get("completed")?,
            position: row.get("position")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            completed_at: row.get("completed_at")?,
            deleted_at: row.get("deleted_at")?,
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DbTag {
    /// UUIDv7.
    pub id: String,
    pub name: String,
    pub color: DbColor,
    pub created_at: DbTimestamp,
    pub updated_at: DbTimestamp,
    pub deleted_at: Option<DbTimestamp>,
}

impl DbTag {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            color: row.get("color")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            deleted_at: row.get("deleted_at")?,
        })
    }

    /// Reads a tag LEFT JOINed to another table, its columns prefixed with `tag_`.
    ///
    /// `None` when the join matched no tag.
    pub fn from_joined_row(row: &Row) -> rusqlite::Result<Option<Self>> {
        let Some(id) = row.get("tag_id")? else {
            return Ok(None);
        };
        Ok(Some(Self {
            id,
            name: row.get("tag_name")?,
            color: row.get("tag_color")?,
            created_at: row.get("tag_created_at")?,
            updated_at: row.get("tag_updated_at")?,
            deleted_at: row.get("tag_deleted_at")?,
        }))
    }
}
