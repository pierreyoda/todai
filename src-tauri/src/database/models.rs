use rusqlite::{Result, Row};

/// Format: YYYY-MM-DD
pub type DbDay = String;
/// Format: 0xRRGGBB (ex. `0xFF00FF`).
pub type DbColor = u32;

/// Unix timestamp, in seconds.
pub type DbTimestamp = i64;

#[derive(Debug)]
pub struct DbTodo {
    /// UUIDv7.
    pub id: String,
    pub day: DbDay,
    pub title: String,
    pub description: Option<String>,
    pub completed: i8,
    /// Fractional index key.
    pub position: String,
    /// Positive; in `estimate_unit`.
    pub estimate: Option<u32>,
    /// `minutes` or `points`; set exactly when `estimate` is.
    pub estimate_unit: Option<String>,
    pub created_at: DbTimestamp,
    pub updated_at: DbTimestamp,
    pub completed_at: Option<DbTimestamp>,
    pub deleted_at: Option<DbTimestamp>,
}

impl DbTodo {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            day: row.get("day")?,
            title: row.get("title")?,
            description: row.get("description")?,
            completed: row.get("completed")?,
            position: row.get("position")?,
            estimate: row.get("estimate")?,
            estimate_unit: row.get("estimate_unit")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            completed_at: row.get("completed_at")?,
            deleted_at: row.get("deleted_at")?,
        })
    }
}

#[derive(Debug)]
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
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            color: row.get("color")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            deleted_at: row.get("deleted_at")?,
        })
    }
}

/// A day's note. Days without a note have no row.
#[derive(Debug)]
pub struct DbNote {
    pub day: DbDay,
    /// Markdown, never empty.
    pub content: String,
    pub created_at: DbTimestamp,
    pub updated_at: DbTimestamp,
}

impl DbNote {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            day: row.get("day")?,
            content: row.get("content")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// A link on a todo.
#[derive(Debug)]
pub struct DbTodoLink {
    /// UUIDv7.
    pub id: String,
    pub todo_id: String,
    /// `github_pull`, `github_issue` or `url`.
    pub kind: String,
    /// Normalized.
    pub url: String,
    /// Set exactly for the GitHub kinds, as are `github_repo` and `github_number`.
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub github_number: Option<u32>,
    pub created_at: DbTimestamp,
}

impl DbTodoLink {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            todo_id: row.get("todo_id")?,
            kind: row.get("kind")?,
            url: row.get("url")?,
            github_owner: row.get("github_owner")?,
            github_repo: row.get("github_repo")?,
            github_number: row.get("github_number")?,
            created_at: row.get("created_at")?,
        })
    }
}

/// A workspace, in the app database.
#[derive(Debug)]
pub struct DbWorkspace {
    /// UUIDv7.
    pub id: String,
    pub name: String,
    /// Absolute and canonical path of its database.
    pub path: String,
    pub created_at: DbTimestamp,
    pub last_opened_at: Option<DbTimestamp>,
    /// Whether a backup is made when it's opened, if its last automatic one is from another day.
    pub auto_backup: bool,
    /// Automatic backups kept: older ones are deleted.
    pub auto_backup_keep: u32,
    /// The GitHub CLI account it uses on github.com, if connected.
    pub github_login: Option<String>,
}

impl DbWorkspace {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            path: row.get("path")?,
            created_at: row.get("created_at")?,
            last_opened_at: row.get("last_opened_at")?,
            auto_backup: row.get("auto_backup")?,
            auto_backup_keep: row.get("auto_backup_keep")?,
            github_login: row.get("github_login")?,
        })
    }
}

/// A backup of a workspace's database, in the app database.
#[derive(Debug)]
pub struct DbWorkspaceBackup {
    /// UUIDv7.
    pub id: String,
    pub workspace_id: String,
    /// Absolute and canonical path of its file.
    pub path: String,
    /// `manual`, `automatic`, `pre_migration` or `pre_restore`.
    pub kind: String,
    pub created_at: DbTimestamp,
}

impl DbWorkspaceBackup {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            workspace_id: row.get("workspace_id")?,
            path: row.get("path")?,
            kind: row.get("kind")?,
            created_at: row.get("created_at")?,
        })
    }
}
