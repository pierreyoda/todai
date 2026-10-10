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
    commands::workspaces::{open_and_activate, workspace, Workspace},
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

/// Restores the backup `id` into its workspace's database, after backing up its current state: restoring this "pre
/// restore" backup undoes it. If its database is missing, it's recreated from the backup instead. Returns the workspace,
/// available again if it wasn't.
///
/// Fails without changing anything if the backup can't be restored (its file missing, not a todai workspace database,
/// or from a newer version), or if its workspace's database is there but can't be opened, so neither backed up first.
#[tauri::command]
pub async fn restore_workspace_backup(state: State<'_, AppState>, id: String) -> Result<Workspace> {
    restore_backup(&state, &id, &Zoned::now())
}

fn restore_backup(state: &AppState, id: &str, now: &Zoned) -> Result<Workspace> {
    let app_db = state.app_db();
    let backup = backups::get(&app_db, id)?;
    let entry = workspaces::get(&app_db, &backup.workspace_id)?;
    let is_active = workspaces::active_id(&app_db)?.as_deref() == Some(entry.id.as_str());

    // The active workspace's open database is restored in place, and stays open. Its lock is released before
    // describing the workspace, which reads it.
    if let Some(mut db) = is_active.then(|| state.db().ok()).flatten() {
        backups::restore(&app_db, state.backups_dir(), &entry, &mut db, &backup, now)?;
    } else {
        let db = if Path::new(&entry.path).exists() {
            let mut db = database::open_existing(&entry.path, &database::WORKSPACE)?;
            backups::restore(&app_db, state.backups_dir(), &entry, &mut db, &backup, now)?;
            db
        } else {
            backups::recreate(&entry, &backup)?
        };
        // The active workspace, whose database could not be opened: usable again.
        if is_active {
            state.replace_db(Some(db));
        }
    }
    log::info!(
        "Restored workspace {:?} from backup {}",
        entry.name,
        backup.path
    );
    workspace(state, &app_db, entry)
}

/// Creates a workspace named `name` (trimmed, not empty) from the backup `id`: its database, at `path`, is a copy of
/// the backup, migrated if it comes from an older version of todai. Then switches to it.
///
/// `path` must be absolute, outside of the backups directory; its directory is created if needed. Fails if anything but
/// an empty file already exists there, or if the backup can't be restored.
#[tauri::command]
pub async fn restore_workspace_backup_as_new(
    state: State<'_, AppState>,
    id: String,
    name: String,
    path: PathBuf,
) -> Result<Workspace> {
    restore_backup_as_new(&state, &id, &name, &path)
}

fn restore_backup_as_new(state: &AppState, id: &str, name: &str, path: &Path) -> Result<Workspace> {
    let app_db = state.app_db();
    let backup = backups::get(&app_db, id)?;
    backups::ensure_outside(state.backups_dir(), path)?;
    let (entry, db) = workspaces::create_from_copy(
        &app_db,
        name,
        path,
        Path::new(&backup.path),
        Timestamp::now().as_second(),
    )?;
    log::info!(
        "Restored backup {} as workspace {:?} at {}",
        backup.path,
        entry.name,
        entry.path
    );
    open_and_activate(state, &app_db, entry, db)
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

    fn now() -> Zoned {
        Zoned::now()
    }

    /// Adds the tag `name` to `entry`'s database.
    fn insert_tag(setup: &Setup, entry: &DbWorkspace, name: &str) {
        let insert = |db: &Connection| {
            db.execute(
                "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES (?1, ?1, 0, 0, 0)",
                [name],
            )
            .unwrap()
        };
        if entry.id == setup.active.id {
            insert(&setup.state.db().unwrap());
        } else {
            insert(&database::open_existing(&entry.path, &database::WORKSPACE).unwrap());
        }
    }

    /// The tags of `entry`'s database.
    fn tag_names(setup: &Setup, entry: &DbWorkspace) -> Vec<String> {
        let app_db = setup.state.app_db();
        with_workspace_db(&setup.state, &app_db, entry, |db| {
            Ok(db
                .prepare("SELECT name FROM tags ORDER BY name")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?)
        })
        .unwrap()
    }

    fn manual_backup(setup: &Setup, entry: &DbWorkspace) -> DbWorkspaceBackup {
        let app_db = setup.state.app_db();
        with_workspace_db(&setup.state, &app_db, entry, |db| {
            backups::create(
                &app_db,
                setup.state.backups_dir(),
                entry,
                db,
                BackupKind::Manual,
                &now(),
            )
        })
        .unwrap()
    }

    #[test]
    fn restore_backup_restores_the_open_database_of_the_active_workspace() {
        let setup = setup();
        insert_tag(&setup, &setup.active, "Before");
        let backup = manual_backup(&setup, &setup.active);
        insert_tag(&setup, &setup.active, "After");

        let restored = restore_backup(&setup.state, &backup.id, &now()).unwrap();
        assert!(restored.is_active && restored.available);
        assert_eq!(tag_names(&setup, &setup.active), ["Before"]);
        // Still the same connection.
        assert_eq!(describe(&setup.state.db().unwrap()).unwrap(), (true, false));
        let kinds: Vec<_> = backups::list(&setup.state.app_db(), &setup.active.id)
            .unwrap()
            .into_iter()
            .map(|backup| backup.kind)
            .collect();
        assert_eq!(kinds, ["pre_restore", "manual"]);
    }

    #[test]
    fn restore_backup_restores_the_database_of_another_workspace() {
        let setup = setup();
        insert_tag(&setup, &setup.other, "Before");
        let backup = manual_backup(&setup, &setup.other);
        insert_tag(&setup, &setup.other, "After");

        let restored = restore_backup(&setup.state, &backup.id, &now()).unwrap();
        assert!(!restored.is_active && restored.available);
        assert_eq!(tag_names(&setup, &setup.other), ["Before"]);
        // The active workspace's is left as is.
        assert_eq!(describe(&setup.state.db().unwrap()).unwrap(), (true, false));
    }

    #[test]
    fn restore_backup_recreates_a_missing_database() {
        let setup = setup();
        insert_tag(&setup, &setup.other, "Before");
        let backup = manual_backup(&setup, &setup.other);
        fs::remove_file(&setup.other.path).unwrap();

        let restored = restore_backup(&setup.state, &backup.id, &now()).unwrap();
        assert!(restored.available);
        assert_eq!(tag_names(&setup, &setup.other), ["Before"]);
    }

    #[test]
    fn restore_backup_makes_the_active_workspace_available_again() {
        let setup = setup();
        insert_tag(&setup, &setup.active, "Before");
        let backup = manual_backup(&setup, &setup.active);
        // Closed, then gone: as if missing when the app started.
        setup.state.replace_db(None);
        fs::remove_file(&setup.active.path).unwrap();

        let restored = restore_backup(&setup.state, &backup.id, &now()).unwrap();
        assert!(restored.is_active && restored.available);
        assert!(setup.state.has_db());
        assert_eq!(tag_names(&setup, &setup.active), ["Before"]);
    }

    #[test]
    fn restore_backup_as_new_switches_to_a_copy() {
        let setup = setup();
        insert_tag(&setup, &setup.active, "Before");
        let backup = manual_backup(&setup, &setup.active);
        let path = setup._dir.path().join("copy.sqlite3");

        let copy = restore_backup_as_new(&setup.state, &backup.id, "Copy", &path).unwrap();
        assert!(copy.is_active && copy.available);
        assert_eq!(copy.name, "Copy");
        // A new connection, to the copy.
        assert_eq!(
            describe(&setup.state.db().unwrap()).unwrap(),
            (false, false)
        );
        let copy = workspaces::get(&setup.state.app_db(), &copy.id).unwrap();
        assert_eq!(tag_names(&setup, &copy), ["Before"]);
        // Its source is left as is.
        assert_eq!(tag_names(&setup, &setup.active), ["Before"]);
    }

    #[test]
    fn restore_backup_as_new_refuses_paths_in_the_backups_directory() {
        let setup = setup();
        let backup = manual_backup(&setup, &setup.active);
        let path = setup.state.backups_dir().join("copy.sqlite3");
        assert!(matches!(
            restore_backup_as_new(&setup.state, &backup.id, "Copy", &path),
            Err(TodaiError::InvalidPath(_))
        ));
        assert!(!path.exists());
    }
}
