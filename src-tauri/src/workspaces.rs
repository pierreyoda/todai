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
pub(crate) fn path_to_db(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| TodaiError::InvalidPath(format!("{} is not valid UTF-8", path.display())))
}

/// Fails if `path` is relative.
pub(crate) fn ensure_absolute(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(TodaiError::InvalidPath(format!(
            "{} is not absolute",
            path.display()
        )));
    }
    Ok(())
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
    ensure_absolute(path)?;
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

/// Fails if `path` is already the database of a registered workspace.
pub(crate) fn ensure_unregistered(app_db: &Connection, path: &str) -> Result<()> {
    if let Some(entry) = find_by_path(app_db, path)? {
        return Err(TodaiError::CommandError(format!(
            "{path} is already the database of workspace {:?}",
            entry.name
        )));
    }
    Ok(())
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

/// Unregisters the workspace `id`, keeping its database. Returns whether it was the active one: there is then no
/// active workspace anymore.
pub fn remove(app_db: &Connection, id: &str) -> Result<bool> {
    let was_active = active_id(app_db)?.as_deref() == Some(id);
    // Explicit, rather than relying on the foreign key's `ON DELETE SET NULL`.
    if was_active {
        app_db
            .prepare_cached("UPDATE app_state SET active_workspace_id = NULL WHERE id = 1")?
            .execute([])?;
    }
    let deleted = app_db
        .prepare_cached("DELETE FROM workspaces WHERE id = ?1")?
        .execute([id])?;
    if deleted == 0 {
        return Err(workspace_not_found(id));
    }
    Ok(was_active)
}

// Both.

/// Opens the database of the registered workspace `entry`.
///
/// Fails if its file is missing.
pub fn open_registered(entry: &DbWorkspace) -> Result<Connection> {
    let path = existing_file(Path::new(&entry.path))?;
    database::open(&path, &database::WORKSPACE)
}

/// Opens the active workspace's database with `open` (e.g. [`open_registered`]), or `None` if there is none or it
/// cannot be opened.
pub fn open_active(
    app_db: &Connection,
    now: DbTimestamp,
    open: impl FnOnce(&DbWorkspace) -> Result<Connection>,
) -> Result<Option<Connection>> {
    let Some(id) = active_id(app_db)? else {
        log::warn!("No active workspace");
        return Ok(None);
    };
    let entry = get(app_db, &id)?;
    match open(&entry) {
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
    ensure_unregistered(app_db, path_db)?;
    let workspace_db = database::open(&path, &database::WORKSPACE)?;
    Ok((insert(app_db, name, path_db, now)?, workspace_db))
}

/// Registers the workspace named `name`, whose database already exists at `path`, and opens it. If it comes from an
/// older version of todai, it's migrated after `before_migration` is called with its registration and its database,
/// opened read-only (e.g. to back it up).
///
/// Fails without writing to it if it's not a todai workspace database (blank ones included: create a workspace there
/// instead), or comes from a newer version; and without registering it if `before_migration` or the migration fails.
pub fn import(
    app_db: &Connection,
    name: &str,
    path: &Path,
    now: DbTimestamp,
    before_migration: impl FnOnce(&DbWorkspace, &Connection) -> Result<()>,
) -> Result<(DbWorkspace, Connection)> {
    let name = validate_name(name)?;
    ensure_absolute(path)?;
    let path = existing_file(path)?;
    let path_db = path_to_db(&path)?;
    ensure_unregistered(app_db, path_db)?;
    let source = database::open_read_only(&path, &database::WORKSPACE)?;
    // Registered first, for `before_migration`, but only kept once opened (rolled back when dropped otherwise).
    let transaction = app_db.unchecked_transaction()?;
    let entry = insert(&transaction, name, path_db, now)?;
    if database::needs_migration(&source, &database::WORKSPACE)? {
        before_migration(&entry, &source)?;
    }
    drop(source);
    let workspace_db = database::open_existing(&path, &database::WORKSPACE)?;
    transaction.commit()?;
    Ok((entry, workspace_db))
}

/// Creates a workspace named `name`, whose new database at `path` is a copy of the workspace database at `source`
/// (e.g. a backup), left untouched, and registers it. The copy is migrated if it comes from an older version of todai.
///
/// Fails if anything but an empty file already exists at `path`, or if `source` is not a todai workspace database or
/// comes from a newer version.
pub fn create_from_copy(
    app_db: &Connection,
    name: &str,
    path: &Path,
    source: &Path,
    now: DbTimestamp,
) -> Result<(DbWorkspace, Connection)> {
    let name = validate_name(name)?;
    let source = database::open_read_only(source, &database::WORKSPACE)?;
    let path = new_file(path)?;
    let path_db = path_to_db(&path)?;
    ensure_unregistered(app_db, path_db)?;
    database::snapshot(&source, &path)?;
    let created = database::open_existing(&path, &database::WORKSPACE)
        .and_then(|workspace_db| Ok((insert(app_db, name, path_db, now)?, workspace_db)));
    if created.is_err() {
        // Best effort: the error worth reporting is the creation's.
        let _ = fs::remove_file(&path);
    }
    created
}

/// Renames the registered workspace `id`. `name` is trimmed, and cannot be empty.
pub fn rename(app_db: &Connection, id: &str, name: &str) -> Result<DbWorkspace> {
    let name = validate_name(name)?;
    let entry = get(app_db, id)?;
    update(app_db, id, name, &entry.path)
}

/// Most automatic backups kept for a workspace.
pub const AUTO_BACKUP_KEEP_MAX: u32 = 100;

/// Sets whether the registered workspace `id` is backed up daily, and how many of its automatic backups are kept: from 1
/// to [`AUTO_BACKUP_KEEP_MAX`]. Extra ones aren't deleted here.
pub fn set_auto_backup(
    app_db: &Connection,
    id: &str,
    auto_backup: bool,
    keep: u32,
) -> Result<DbWorkspace> {
    if !(1..=AUTO_BACKUP_KEEP_MAX).contains(&keep) {
        return Err(TodaiError::CommandError(format!(
            "From 1 to {AUTO_BACKUP_KEEP_MAX} automatic backups can be kept, not {keep}"
        )));
    }
    app_db
        .prepare_cached(
            "UPDATE workspaces SET auto_backup = :auto_backup, auto_backup_keep = :keep
             WHERE id = :id
             RETURNING *",
        )?
        .query_row(
            named_params! { ":id": id, ":auto_backup": auto_backup, ":keep": keep },
            DbWorkspace::from_row,
        )
        .optional()?
        .ok_or_else(|| workspace_not_found(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_791_000_000;

    /// For `import`, without backing up databases before migrating them.
    fn no_backup(_: &DbWorkspace, _: &Connection) -> Result<()> {
        Ok(())
    }

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
        assert!(open_active(&setup.app_db, NOW, open_registered)
            .unwrap()
            .is_none());

        let work = setup.create("Work", "work.sqlite3");
        set_active(&setup.app_db, &work.id, NOW).unwrap();
        assert!(open_active(&setup.app_db, NOW + 60, open_registered)
            .unwrap()
            .is_some());
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
        assert!(open_active(&setup.app_db, NOW, open_registered)
            .unwrap()
            .is_none());
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
    fn import_registers_an_existing_workspace() {
        let setup = Setup::new();
        let path = setup.path("work.sqlite3");
        database::open(&path, &database::WORKSPACE).unwrap();

        let (work, _) = import(&setup.app_db, "  Work ", &path, NOW, no_backup).unwrap();
        assert_eq!(work.name, "Work");
        assert_eq!(work.path, path.to_str().unwrap());
        assert_eq!(get(&setup.app_db, &work.id).unwrap().name, "Work");
        // Not activated.
        assert_eq!(active_id(&setup.app_db).unwrap(), None);
    }

    #[test]
    fn import_refuses_an_already_registered_workspace() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        assert!(import(
            &setup.app_db,
            "Again",
            Path::new(&work.path),
            NOW,
            no_backup
        )
        .is_err());
        assert_eq!(list(&setup.app_db).unwrap().len(), 1);
    }

    #[test]
    fn import_refuses_invalid_paths_and_databases() {
        let setup = Setup::new();
        let workspace = setup.path("work.sqlite3");
        database::open(&workspace, &database::WORKSPACE).unwrap();
        assert!(import(&setup.app_db, " ", &workspace, NOW, no_backup).is_err());
        assert!(import(
            &setup.app_db,
            "Work",
            Path::new("work.sqlite3"),
            NOW,
            no_backup
        )
        .is_err());

        let missing = setup.path("missing.sqlite3");
        assert!(matches!(
            import(&setup.app_db, "Work", &missing, NOW, no_backup),
            Err(TodaiError::WorkspaceUnavailable(_))
        ));
        assert!(!missing.exists());

        let empty = setup.path("empty.sqlite3");
        fs::write(&empty, "").unwrap();
        let text = setup.path("notes.txt");
        fs::write(&text, "Not a database, but long enough to have a header.".repeat(10)).unwrap();
        let app = setup.path("app.sqlite3");
        database::open(&app, &database::APP).unwrap();
        for path in [&empty, &text, &app] {
            assert!(matches!(
                import(&setup.app_db, "Work", path, NOW, no_backup),
                Err(TodaiError::InvalidDatabase(_))
            ));
        }
        assert_eq!(fs::metadata(&empty).unwrap().len(), 0);
        assert!(list(&setup.app_db).unwrap().is_empty());
    }

    /// The schema version of the workspace database at `path`.
    fn version(path: &Path) -> i64 {
        database::open_read_only(path, &database::WORKSPACE)
            .unwrap()
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }

    fn first_version_workspace(path: &Path) {
        database::create_first_version_test_database(path, &database::WORKSPACE);
    }

    #[test]
    fn import_calls_before_migration_only_for_outdated_databases() {
        let setup = Setup::new();
        let old = setup.path("old.sqlite3");
        first_version_workspace(&old);
        let current = setup.path("current.sqlite3");
        database::open(&current, &database::WORKSPACE).unwrap();

        let mut called = None;
        let (imported, _) = import(&setup.app_db, "Old", &old, NOW, |entry, source| {
            let version: i64 = source.pragma_query_value(None, "user_version", |r| r.get(0))?;
            called = Some((entry.id.clone(), version));
            Ok(())
        })
        .unwrap();
        // Registered, and before migrating.
        assert_eq!(called, Some((imported.id, 1)));
        assert!(version(&old) > 1);

        import(&setup.app_db, "Current", &current, NOW, |_, _| {
            panic!("Called for an up-to-date database")
        })
        .unwrap();
    }

    #[test]
    fn import_registers_nothing_when_before_migration_fails() {
        let setup = Setup::new();
        let old = setup.path("old.sqlite3");
        first_version_workspace(&old);

        let failed = import(&setup.app_db, "Old", &old, NOW, |_, _| {
            Err(TodaiError::CommandError("Disk full".into()))
        });
        assert!(failed.is_err());
        assert!(list(&setup.app_db).unwrap().is_empty());
        // Not migrated.
        assert_eq!(version(&old), 1);
    }

    #[test]
    fn set_auto_backup_updates_and_validates() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        let updated = set_auto_backup(&setup.app_db, &work.id, false, 3).unwrap();
        assert!(!updated.auto_backup);
        assert_eq!(updated.auto_backup_keep, 3);
        assert!(!get(&setup.app_db, &work.id).unwrap().auto_backup);

        for keep in [0, AUTO_BACKUP_KEEP_MAX + 1] {
            assert!(set_auto_backup(&setup.app_db, &work.id, true, keep).is_err());
        }
        assert!(set_auto_backup(&setup.app_db, "missing", true, 3).is_err());
        assert_eq!(get(&setup.app_db, &work.id).unwrap().auto_backup_keep, 3);
    }

    #[test]
    fn create_from_copy_registers_a_copy_of_the_source() {
        let setup = Setup::new();
        let source = setup.path("backup.sqlite3");
        database::open(&source, &database::WORKSPACE)
            .unwrap()
            .execute(
                "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES ('t', 'Work', 0, 0, 0)",
                [],
            )
            .unwrap();
        let source_bytes = fs::read(&source).unwrap();

        let (copy, db) = create_from_copy(
            &setup.app_db,
            " Restored ",
            &setup.path("a/restored.sqlite3"),
            &source,
            NOW,
        )
        .unwrap();
        assert_eq!(copy.name, "Restored");
        assert_eq!(
            copy.path,
            setup.path("a/restored.sqlite3").to_str().unwrap()
        );
        let name: String = db
            .query_row("SELECT name FROM tags", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Work");
        assert_eq!(get(&setup.app_db, &copy.id).unwrap().name, "Restored");
        assert_eq!(fs::read(&source).unwrap(), source_bytes);
        // Not activated.
        assert_eq!(active_id(&setup.app_db).unwrap(), None);
    }

    #[test]
    fn create_from_copy_refuses_invalid_sources_and_targets() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        let app = setup.path("app.sqlite3");
        database::open(&app, &database::APP).unwrap();
        let existing = setup.path("existing.sqlite3");
        fs::write(&existing, "data").unwrap();

        let target = setup.path("copy.sqlite3");
        for source in [&app, &setup.path("missing.sqlite3")] {
            assert!(create_from_copy(&setup.app_db, "Copy", &target, source, NOW).is_err());
        }
        assert!(!target.exists());
        let source = Path::new(&work.path);
        for target in [existing.as_path(), Path::new(&work.path)] {
            assert!(create_from_copy(&setup.app_db, "Copy", target, source, NOW).is_err());
        }
        assert_eq!(fs::read_to_string(&existing).unwrap(), "data");
        assert_eq!(list(&setup.app_db).unwrap().len(), 1);
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
    fn remove_keeps_the_database() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        let personal = setup.create("Personal", "personal.sqlite3");
        set_active(&setup.app_db, &personal.id, NOW).unwrap();

        assert!(!remove(&setup.app_db, &work.id).unwrap());
        assert!(Path::new(&work.path).is_file());
        let workspaces = list(&setup.app_db).unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].id, personal.id);
        assert_eq!(active_id(&setup.app_db).unwrap(), Some(personal.id));
        assert!(remove(&setup.app_db, &work.id).is_err());
    }

    #[test]
    fn remove_clears_the_active_workspace() {
        let setup = Setup::new();
        let work = setup.create("Work", "work.sqlite3");
        set_active(&setup.app_db, &work.id, NOW).unwrap();

        assert!(remove(&setup.app_db, &work.id).unwrap());
        assert!(Path::new(&work.path).is_file());
        assert!(list(&setup.app_db).unwrap().is_empty());
        assert_eq!(active_id(&setup.app_db).unwrap(), None);
        assert!(open_active(&setup.app_db, NOW, open_registered)
            .unwrap()
            .is_none());
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
