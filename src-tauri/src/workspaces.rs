//! Each workspace has its own database, and is listed in the app database.

use rusqlite::{named_params, Connection, OptionalExtension};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    database::{
        self,
        models::{DbTimestamp, DbWorkspace},
    },
    errors::{Result, TodaiError},
};

/// Application database filename, in the app data directory.
pub const APP_DATABASE_FILE_NAME: &str = "app.sqlite3";

/// Trims `name`, which cannot be empty.
pub fn validate_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(TodaiError::CommandError(
            "Workspace name cannot be empty".into(),
        ));
    }
    Ok(name)
}

fn workspace_not_found(id: &str) -> TodaiError {
    TodaiError::CommandError(format!("Workspace {id} not found"))
}

/// `path`, as stored in the app database.
fn path_to_db(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| TodaiError::InvalidPath(format!("{} is not valid UTF-8", path.display())))
}

/// `path`, canonical, if a file exists there.
fn existing_file(path: &Path) -> Result<PathBuf> {
    if !path.is_file() {
        return Err(TodaiError::WorkspaceUnavailable(path.display().to_string()));
    }
    Ok(fs::canonicalize(path)?)
}

/// `path`, canonical and with its directory created, to create a database at.
///
/// Fails if anything but an empty file already exists there.
fn new_file(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(TodaiError::InvalidPath(format!(
            "{} is not absolute",
            path.display()
        )));
    }
    let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) else {
        return Err(TodaiError::InvalidPath(format!(
            "{} is not a file path",
            path.display()
        )));
    };
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() && metadata.len() == 0 => {}
        Ok(_) => {
            return Err(TodaiError::CommandError(format!(
                "A file already exists at {}: open it as an existing workspace instead",
                path.display()
            )))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    fs::create_dir_all(parent)?;
    Ok(fs::canonicalize(parent)?.join(file_name))
}

// App database.

/// The registered workspaces, sorted by name.
pub fn list(app_db: &Connection) -> Result<Vec<DbWorkspace>> {
    let workspaces = app_db
        .prepare_cached("SELECT * FROM workspaces ORDER BY name COLLATE NOCASE, created_at")?
        .query_map([], DbWorkspace::from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(workspaces)
}

fn find(app_db: &Connection, id: &str) -> Result<Option<DbWorkspace>> {
    Ok(app_db
        .prepare_cached("SELECT * FROM workspaces WHERE id = ?1")?
        .query_row([id], DbWorkspace::from_row)
        .optional()?)
}

fn find_by_path(app_db: &Connection, path: &str) -> Result<Option<DbWorkspace>> {
    Ok(app_db
        .prepare_cached("SELECT * FROM workspaces WHERE path = ?1")?
        .query_row([path], DbWorkspace::from_row)
        .optional()?)
}

/// The registered workspace `id`.
pub fn get(app_db: &Connection, id: &str) -> Result<DbWorkspace> {
    find(app_db, id)?.ok_or_else(|| workspace_not_found(id))
}

pub fn active_id(app_db: &Connection) -> Result<Option<String>> {
    Ok(app_db
        .prepare_cached("SELECT active_workspace_id FROM app_state WHERE id = 1")?
        .query_row([], |row| row.get(0))?)
}

/// Registers the workspace `name`, whose database is at `path`, created at `now`.
fn insert(app_db: &Connection, name: &str, path: &str, now: DbTimestamp) -> Result<DbWorkspace> {
    Ok(app_db
        .prepare_cached(
            "INSERT INTO workspaces (id, name, path, created_at)
             VALUES (:id, :name, :path, :now)
             RETURNING *",
        )?
        .query_row(
            named_params! {
                ":id": Uuid::now_v7().to_string(),
                ":name": name,
                ":path": path,
                ":now": now,
            },
            DbWorkspace::from_row,
        )?)
}

fn update(app_db: &Connection, id: &str, name: &str, path: &str) -> Result<DbWorkspace> {
    app_db
        .prepare_cached(
            "UPDATE workspaces SET name = :name, path = :path WHERE id = :id RETURNING *",
        )?
        .query_row(
            named_params! { ":id": id, ":name": name, ":path": path },
            DbWorkspace::from_row,
        )
        .optional()?
        .ok_or_else(|| workspace_not_found(id))
}

/// Makes the registered workspace `id` the active one, opened at `now`.
pub fn set_active(app_db: &Connection, id: &str, now: DbTimestamp) -> Result<()> {
    let updated = app_db
        .prepare_cached("UPDATE workspaces SET last_opened_at = :now WHERE id = :id")?
        .execute(named_params! { ":id": id, ":now": now })?;
    if updated == 0 {
        return Err(workspace_not_found(id));
    }
    app_db
        .prepare_cached("UPDATE app_state SET active_workspace_id = ?1 WHERE id = 1")?
        .execute([id])?;
    Ok(())
}

/// Unregisters the workspace `id`, which cannot be the active one. Its database is kept.
pub fn remove(app_db: &Connection, id: &str) -> Result<()> {
    if active_id(app_db)?.as_deref() == Some(id) {
        return Err(TodaiError::CommandError(
            "The active workspace cannot be removed: switch to another one first".into(),
        ));
    }
    let deleted = app_db
        .prepare_cached("DELETE FROM workspaces WHERE id = ?1")?
        .execute([id])?;
    if deleted == 0 {
        return Err(workspace_not_found(id));
    }
    Ok(())
}

// Both.

/// Opens the database of the registered workspace `entry`.
///
/// Fails if its file is missing.
pub fn open_registered(entry: &DbWorkspace) -> Result<Connection> {
    let path = existing_file(Path::new(&entry.path))?;
    database::open(&path, &database::WORKSPACE)
}

/// Opens the active workspace's database, or `None` if there is none or it cannot be opened.
pub fn open_active(app_db: &Connection, now: DbTimestamp) -> Result<Option<Connection>> {
    let Some(id) = active_id(app_db)? else {
        log::warn!("No active workspace");
        return Ok(None);
    };
    let entry = get(app_db, &id)?;
    match open_registered(&entry) {
        Ok(workspace_db) => {
            log::info!("Opened workspace {:?} at {}", entry.name, entry.path);
            set_active(app_db, &id, now)?;
            Ok(Some(workspace_db))
        }
        Err(error) => {
            log::warn!("Cannot open workspace {:?}: {error}", entry.name);
            Ok(None)
        }
    }
}

/// Creates a workspace named `name`, with a new database at `path`, and registers it.
///
/// Fails if anything but an empty file already exists at `path`.
pub fn create(
    app_db: &Connection,
    name: &str,
    path: &Path,
    now: DbTimestamp,
) -> Result<(DbWorkspace, Connection)> {
    let name = validate_name(name)?;
    let path = new_file(path)?;
    let path_db = path_to_db(&path)?;
    if let Some(entry) = find_by_path(app_db, path_db)? {
        return Err(TodaiError::CommandError(format!(
            "{path_db} is already the database of workspace {:?}",
            entry.name
        )));
    }
    let workspace_db = database::open(&path, &database::WORKSPACE)?;
    Ok((insert(app_db, name, path_db, now)?, workspace_db))
}

/// Renames the registered workspace `id`. `name` is trimmed, and cannot be empty.
pub fn rename(app_db: &Connection, id: &str, name: &str) -> Result<DbWorkspace> {
    let name = validate_name(name)?;
    let entry = get(app_db, id)?;
    update(app_db, id, name, &entry.path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_791_000_000;

    struct Setup {
        dir: tempfile::TempDir,
        app_db: Connection,
    }

    impl Setup {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().unwrap(),
                app_db: database::open_test_database(&database::APP),
            }
        }

        /// The canonical path of `name` in the temporary directory, as in the app database.
        fn path(&self, name: &str) -> PathBuf {
            fs::canonicalize(self.dir.path()).unwrap().join(name)
        }

        fn create(&self, name: &str, file_name: &str) -> DbWorkspace {
            create(&self.app_db, name, &self.path(file_name), NOW)
                .unwrap()
                .0
        }
    }

    #[test]
    fn open_active_opens_the_active_workspace() {
        let setup = Setup::new();
        assert!(open_active(&setup.app_db, NOW).unwrap().is_none());

        let work = setup.create("Work", "work.sqlite3");
        set_active(&setup.app_db, &work.id, NOW).unwrap();
        assert!(open_active(&setup.app_db, NOW + 60).unwrap().is_some());
        assert_eq!(
            get(&setup.app_db, &work.id).unwrap().last_opened_at,
            Some(NOW + 60)
        );
    }

    #[test]
    fn open_active_is_none_when_the_database_is_missing() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        set_active(&setup.app_db, &work.id, NOW).unwrap();
        fs::remove_file(&work.path).unwrap();
        assert!(open_active(&setup.app_db, NOW).unwrap().is_none());
    }

    #[test]
    fn create_registers_the_workspace() {
        let setup = Setup::new();
        let (work, _) = create(
            &setup.app_db,
            "  Work ",
            &setup.path("a/b/work.sqlite3"),
            NOW,
        )
        .unwrap();
        assert_eq!(work.name, "Work");
        assert_eq!(work.path, setup.path("a/b/work.sqlite3").to_str().unwrap());
        assert_eq!(get(&setup.app_db, &work.id).unwrap().name, "Work");
        // Not activated.
        assert_eq!(active_id(&setup.app_db).unwrap(), None);
    }

    #[test]
    fn create_refuses_existing_files_and_invalid_paths() {
        let setup = Setup::new();
        let existing = setup.path("existing.sqlite3");
        fs::write(&existing, "data").unwrap();
        assert!(create(&setup.app_db, "Work", &existing, NOW).is_err());
        assert_eq!(fs::read_to_string(&existing).unwrap(), "data");

        assert!(create(&setup.app_db, "Work", &setup.path(""), NOW).is_err());
        assert!(create(&setup.app_db, "Work", Path::new("relative.sqlite3"), NOW).is_err());
        assert!(create(&setup.app_db, "   ", &setup.path("blank.sqlite3"), NOW).is_err());
        assert!(list(&setup.app_db).unwrap().is_empty());
    }

    #[test]
    fn create_accepts_an_empty_file() {
        let setup = Setup::new();
        fs::write(setup.path("empty.sqlite3"), "").unwrap();
        assert_eq!(setup.create("Work", "empty.sqlite3").name, "Work");
    }

    #[test]
    fn rename_trims_and_refuses_an_empty_name() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        let renamed = rename(&setup.app_db, &work.id, " Job ").unwrap();
        assert_eq!(renamed.name, "Job");
        assert_eq!(get(&setup.app_db, &work.id).unwrap().name, "Job");
        assert!(rename(&setup.app_db, &work.id, " ").is_err());
    }

    #[test]
    fn remove_keeps_the_database_and_refuses_the_active_workspace() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        let personal = setup.create("Personal", "personal.sqlite3");
        set_active(&setup.app_db, &personal.id, NOW).unwrap();

        assert!(remove(&setup.app_db, &personal.id).is_err());
        remove(&setup.app_db, &work.id).unwrap();
        assert!(Path::new(&work.path).is_file());
        let workspaces = list(&setup.app_db).unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].id, personal.id);
        assert!(remove(&setup.app_db, &work.id).is_err());
    }

    #[test]
    fn list_sorts_by_name_regardless_of_case() {
        let setup = Setup::new();
        setup.create("work", "work.sqlite3");
        setup.create("Personal", "personal.sqlite3");
        setup.create("Archive", "archive.sqlite3");
        let names: Vec<String> = list(&setup.app_db)
            .unwrap()
            .into_iter()
            .map(|workspace| workspace.name)
            .collect();
        assert_eq!(names, ["Archive", "Personal", "work"]);
    }
}
