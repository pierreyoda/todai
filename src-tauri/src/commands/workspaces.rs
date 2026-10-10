use jiff::{Timestamp, Zoned};
use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::{
    backups,
    database::models::{DbTimestamp, DbWorkspace},
    errors::Result,
    state::AppState,
    workspaces,
};

/// A workspace, with its own database.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    /// Absolute path of its database.
    pub path: String,
    pub is_active: bool,
    /// Whether its database can be used: open for the active workspace, existing for the others.
    pub available: bool,
    pub created_at: Timestamp,
    pub last_opened_at: Option<Timestamp>,
    /// When its last backup was made, whatever its kind.
    pub last_backup_at: Option<Timestamp>,
    /// Whether a backup is made when it's opened, if its last automatic one is from another day.
    pub auto_backup: bool,
    /// Automatic backups kept: older ones are deleted.
    pub auto_backup_keep: u32,
}

impl Workspace {
    fn new(
        entry: DbWorkspace,
        is_active: bool,
        available: bool,
        last_backup_at: Option<DbTimestamp>,
    ) -> Result<Self> {
        Ok(Self {
            id: entry.id,
            name: entry.name,
            path: entry.path,
            is_active,
            available,
            created_at: Timestamp::from_second(entry.created_at)?,
            last_opened_at: entry
                .last_opened_at
                .map(Timestamp::from_second)
                .transpose()?,
            last_backup_at: last_backup_at.map(Timestamp::from_second).transpose()?,
            auto_backup: entry.auto_backup,
            auto_backup_keep: entry.auto_backup_keep,
        })
    }
}

/// `entry`, as seen by the frontend. `app_db` is `state`'s, already locked.
pub(crate) fn workspace(
    state: &AppState,
    app_db: &Connection,
    entry: DbWorkspace,
) -> Result<Workspace> {
    let is_active = workspaces::active_id(app_db)?.as_deref() == Some(entry.id.as_str());
    let available = if is_active {
        state.has_db()
    } else {
        Path::new(&entry.path).is_file()
    };
    let last_backup_at = backups::last_created_at(app_db, &entry.id)?;
    Workspace::new(entry, is_active, available, last_backup_at)
}

/// Makes `entry`, whose database is `db`, the active workspace.
pub(crate) fn open_and_activate(
    state: &AppState,
    app_db: &Connection,
    entry: DbWorkspace,
    db: Connection,
) -> Result<Workspace> {
    workspaces::set_active(app_db, &entry.id, Timestamp::now().as_second())?;
    state.replace_db(Some(db));
    log::info!("Switched to workspace {:?} at {}", entry.name, entry.path);
    // With its new `last_opened_at`.
    let entry = workspaces::get(app_db, &entry.id)?;
    workspace(state, app_db, entry)
}

/// Lists the workspaces, sorted by name.
#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>> {
    let app_db = state.app_db();
    workspaces::list(&app_db)?
        .into_iter()
        .map(|entry| workspace(&state, &app_db, entry))
        .collect()
}

/// The active workspace, if any. If its database could not be opened, it's not `available`.
#[tauri::command]
pub async fn get_active_workspace(state: State<'_, AppState>) -> Result<Option<Workspace>> {
    let app_db = state.app_db();
    workspaces::active_id(&app_db)?
        .map(|id| {
            let entry = workspaces::get(&app_db, &id)?;
            workspace(&state, &app_db, entry)
        })
        .transpose()
}

/// Creates a workspace named `name` (trimmed, not empty), with a new database at `path`, and switches to it if
/// `activate`.
///
/// `path` must be absolute, outside of the backups directory; its directory is created if needed. Fails if anything but
/// an empty file already exists there.
#[tauri::command]
pub async fn create_workspace(
    state: State<'_, AppState>,
    name: String,
    path: PathBuf,
) -> Result<Workspace> {
    let app_db = state.app_db();
    backups::ensure_outside(state.backups_dir(), &path)?;
    let (entry, db) = workspaces::create(&app_db, &name, &path, Timestamp::now().as_second())?;
    open_and_activate(&state, &app_db, entry, db)
}

/// Registers the workspace named `name` (trimmed, not empty), whose database already exists at `path`, and switches
/// to it. Its database is migrated if it comes from an older version of todai, after backing it up.
///
/// `path` must be absolute, outside of the backups directory: a backup is restored as a new workspace instead. Fails
/// without writing to it if it's not a todai workspace database (blank ones included), comes from a newer version, is
/// already registered, or can't be backed up before migrating it.
#[tauri::command]
pub async fn import_workspace(
    state: State<'_, AppState>,
    name: String,
    path: PathBuf,
) -> Result<Workspace> {
    let app_db = state.app_db();
    backups::ensure_outside(state.backups_dir(), &path)?;
    let now = Zoned::now();
    let (entry, db) = workspaces::import(
        &app_db,
        &name,
        &path,
        now.timestamp().as_second(),
        |entry, source| {
            backups::back_up_before_migration(&app_db, state.backups_dir(), entry, source, &now)
        },
    )?;
    open_and_activate(&state, &app_db, entry, db)
}

/// Switches to the workspace `id`, closing the previous one's database. It's backed up before being migrated, if it
/// comes from an older version of todai, then daily.
///
/// Fails if its database cannot be opened, or backed up before migrating it.
#[tauri::command]
pub async fn switch_to_workspace(state: State<'_, AppState>, id: String) -> Result<Workspace> {
    let app_db = state.app_db();
    let entry = workspaces::get(&app_db, &id)?;
    let db = backups::open_workspace(&app_db, state.backups_dir(), &entry, &Zoned::now())?;
    open_and_activate(&state, &app_db, entry, db)
}

/// Renames the workspace `id`. `name` is trimmed, and cannot be empty.
#[tauri::command]
pub async fn rename_workspace(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Workspace> {
    let app_db = state.app_db();
    let entry = workspaces::rename(&app_db, &id, &name)?;
    workspace(&state, &app_db, entry)
}

/// Unregisters the workspace `id`, keeping its database: it can be imported again.
///
/// If it was the active one, its database is closed: there is no active workspace anymore, until switching to another
/// one (or creating or importing one).
#[tauri::command]
pub async fn remove_workspace(state: State<'_, AppState>, id: String) -> Result<()> {
    let app_db = state.app_db();
    if workspaces::remove(&app_db, &id)? {
        log::info!("Removed the active workspace {id}: no workspace is open anymore");
        state.replace_db(None);
    }
    Ok(())
}
