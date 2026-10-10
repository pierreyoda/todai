use jiff::{Timestamp, Zoned};
use rusqlite::Connection;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    backups::{self, BackupKind},
    database::{
        self,
        models::{DbWorkspace, DbWorkspaceBackup},
    },
    errors::{Result, TodaiError},
    state::AppState,
    workspaces,
};

/// A backup of a workspace's database.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBackup {
    pub id: String,
    pub workspace_id: String,
    /// Absolute path of its file.
    pub path: String,
    pub kind: BackupKind,
    /// Whether its file still exists (e.g. not deleted by hand).
    pub available: bool,
    /// Of its file, in bytes, if available.
    pub size: Option<u64>,
    pub created_at: Timestamp,
}

/// With the size of its file, read from the disk.
impl TryFrom<DbWorkspaceBackup> for WorkspaceBackup {
    type Error = TodaiError;

    fn try_from(backup: DbWorkspaceBackup) -> std::result::Result<Self, Self::Error> {
        let size = fs::metadata(&backup.path)
            .ok()
            .filter(|metadata| metadata.is_file())
            .map(|metadata| metadata.len());
        Ok(Self {
            kind: backup.kind.as_str().try_into()?,
            available: size.is_some(),
            size,
            created_at: Timestamp::from_second(backup.created_at)?,
            id: backup.id,
            workspace_id: backup.workspace_id,
            path: backup.path,
        })
    }
}

/// Runs `f` with `entry`'s database: the open one for the active workspace, else its file, opened read-only. `app_db`
/// is `state`'s, already locked.
///
/// Fails if its database is unavailable.
fn with_workspace_db<T>(
    state: &AppState,
    app_db: &Connection,
    entry: &DbWorkspace,
    f: impl FnOnce(&Connection) -> Result<T>,
) -> Result<T> {
    if workspaces::active_id(app_db)?.as_deref() == Some(entry.id.as_str()) {
        // Can't be replaced meanwhile: switching workspaces locks `app_db` first.
        if let Ok(db) = state.db() {
            return f(&db);
        }
    }
    if !Path::new(&entry.path).is_file() {
        return Err(TodaiError::WorkspaceUnavailable(entry.path.clone()));
    }
    f(&database::open_read_only(
        &entry.path,
        &database::WORKSPACE,
    )?)
}

/// The backups of the workspace `workspace_id`, most recent first.
#[tauri::command]
pub async fn list_workspace_backups(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<WorkspaceBackup>> {
    let app_db = state.app_db();
    // An unknown workspace fails, rather than having no backups.
    workspaces::get(&app_db, &workspace_id)?;
    backups::list(&app_db, &workspace_id)?
        .into_iter()
        .map(WorkspaceBackup::try_from)
        .collect()
}

/// Backs up the workspace `workspace_id`, at the user's request. Fails if its database is unavailable.
#[tauri::command]
pub async fn create_workspace_backup(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<WorkspaceBackup> {
    let app_db = state.app_db();
    let entry = workspaces::get(&app_db, &workspace_id)?;
    let backup = with_workspace_db(&state, &app_db, &entry, |db| {
        backups::create(
            &app_db,
            state.backups_dir(),
            &entry,
            db,
            BackupKind::Manual,
            &Zoned::now(),
        )
    })?;
    log::info!("Backed up workspace {:?} to {}", entry.name, backup.path);
    backup.try_into()
}

/// Deletes the backup `id`, with its file.
#[tauri::command]
pub async fn delete_workspace_backup(state: State<'_, AppState>, id: String) -> Result<()> {
    backups::delete(&state.app_db(), &id)?;
    log::info!("Deleted backup {id}");
    Ok(())
}

/// Writes a copy of the workspace `workspace_id`'s database to `path`, absolute: unlike a backup, it's not listed. A
/// file there is replaced, unless it's a workspace's database or a backup. Fails if its database is unavailable.
#[tauri::command]
pub async fn export_workspace(
    state: State<'_, AppState>,
    workspace_id: String,
    path: PathBuf,
) -> Result<()> {
    let app_db = state.app_db();
    let entry = workspaces::get(&app_db, &workspace_id)?;
    with_workspace_db(&state, &app_db, &entry, |db| {
        backups::export(&app_db, state.backups_dir(), db, &path)
    })?;
    log::info!("Exported workspace {:?} to {}", entry.name, path.display());
    Ok(())
}

/// Opens the directory of the workspace `workspace_id`'s backups in the file manager, created if needed.
#[tauri::command]
pub async fn open_workspace_backups_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<()> {
    workspaces::get(&state.app_db(), &workspace_id)?;
    let dir = backups::workspace_dir(state.backups_dir(), &workspace_id);
    fs::create_dir_all(&dir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|error| {
            TodaiError::CommandError(format!("Cannot open {}: {error}", dir.display()))
        })
}

#[cfg(test)]
mod tests {
    use rusqlite::MAIN_DB;

    use super::*;

    const NOW: i64 = 1_791_000_000;

    struct Setup {
        _dir: tempfile::TempDir,
        state: AppState,
        active: DbWorkspace,
        other: DbWorkspace,
    }

    /// Two workspaces: the active one, open, and another one.
    fn setup() -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let app_db = database::open_test_database(&database::APP);
        let (active, db) =
            workspaces::create(&app_db, "Active", &dir.path().join("active.sqlite3"), NOW).unwrap();
        let (other, _) =
            workspaces::create(&app_db, "Other", &dir.path().join("other.sqlite3"), NOW).unwrap();
        workspaces::set_active(&app_db, &active.id, NOW).unwrap();
        // Only visible from this connection.
        db.execute_batch("CREATE TEMP TABLE marker (id INTEGER)")
            .unwrap();
        let backups_dir = dir.path().join(backups::BACKUPS_DIR_NAME);
        Setup {
            _dir: dir,
            state: AppState::new(app_db, Some(db), backups_dir),
            active,
            other,
        }
    }

    /// Whether `conn` is the active workspace's open connection, and whether it's read-only.
    fn describe(conn: &Connection) -> Result<(bool, bool)> {
        let open = conn
            .query_row(
                "SELECT COUNT(*) FROM temp.sqlite_schema WHERE name = 'marker'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|count| count == 1)?;
        Ok((open, conn.is_readonly(MAIN_DB)?))
    }

    #[test]
    fn with_workspace_db_uses_the_open_database_of_the_active_workspace() {
        let setup = setup();
        let app_db = setup.state.app_db();
        let described = with_workspace_db(&setup.state, &app_db, &setup.active, describe);
        assert_eq!(described.unwrap(), (true, false));
    }

    #[test]
    fn with_workspace_db_reads_the_others_read_only() {
        let setup = setup();
        let app_db = setup.state.app_db();
        let described = with_workspace_db(&setup.state, &app_db, &setup.other, describe);
        assert_eq!(described.unwrap(), (false, true));
    }

    #[test]
    fn with_workspace_db_fails_without_a_database_file() {
        let setup = setup();
        fs::remove_file(&setup.other.path).unwrap();
        let app_db = setup.state.app_db();
        assert!(matches!(
            with_workspace_db(&setup.state, &app_db, &setup.other, describe),
            Err(TodaiError::WorkspaceUnavailable(_))
        ));
        assert!(!Path::new(&setup.other.path).exists());
    }
}
