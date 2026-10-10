use rusqlite::{
    backup::{Backup, StepResult},
    Connection, ErrorCode, OpenFlags,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::errors::{Result, TodaiError};

pub mod models;

/// A kind of todai database, with its own migrations.
pub struct DatabaseKind {
    name: &'static str,
    /// Stored in `PRAGMA application_id`, so that a database is never opened as another kind.
    ///
    /// Four ASCII letters, one per byte: "TD" for todai, then the kind.
    application_id: i32,
    migrations: &'static [&'static str],
}

/// A workspace's todos, tags and notes.
pub const WORKSPACE: DatabaseKind = DatabaseKind {
    name: "workspace",
    // "TDWS" (ToDai WorkSpace).
    application_id: 0x5444_5753,
    migrations: &[
        include_str!("../migrations/001_init.sql"),
        include_str!("../migrations/002_tags.sql"),
        include_str!("../migrations/003_tags_non_unique_name.sql"),
        include_str!("../migrations/004_todo_estimate.sql"),
        include_str!("../migrations/005_notes.sql"),
    ],
};

/// The workspaces, shared by the whole app.
pub const APP: DatabaseKind = DatabaseKind {
    name: "app",
    // "TDAP" (ToDai APp).
    application_id: 0x5444_4150,
    migrations: &[
        include_str!("../migrations-app/001_init.sql"),
        include_str!("../migrations-app/002_workspace_backups.sql"),
    ],
};

/// Opens (or creates) the `kind` database at `path`, and migrates it.
///
/// Fails without writing to it if it's not a `kind` database, or comes from a newer version of todai.
pub fn open(path: impl AsRef<Path>, kind: &DatabaseKind) -> Result<Connection> {
    let path = path.as_ref();
    let conn = Connection::open(path)?;
    check_kind(&conn, path, kind, true)?;
    init(conn, kind)
}

/// Opens the existing `kind` database at `path`, and migrates it if it comes from an older version of todai.
///
/// Unlike [`open`], fails without writing to it if it's blank (an empty file, or SQLite without any table), and
/// without creating it if it's missing.
pub fn open_existing(path: impl AsRef<Path>, kind: &DatabaseKind) -> Result<Connection> {
    let path = path.as_ref();
    let flags = OpenFlags::default().difference(OpenFlags::SQLITE_OPEN_CREATE);
    let conn = Connection::open_with_flags(path, flags)?;
    check_kind(&conn, path, kind, false)?;
    init(conn, kind)
}

/// Opens the existing `kind` database at `path` read-only: neither created, migrated nor otherwise written to, e.g. to
/// copy it.
///
/// Fails if it's not a `kind` database (blank ones included), or comes from a newer version of todai.
pub fn open_read_only(path: impl AsRef<Path>, kind: &DatabaseKind) -> Result<Connection> {
    let path = path.as_ref();
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(path, flags)?;
    check_kind(&conn, path, kind, false)?;
    Ok(conn)
}

/// Whether `conn`'s `kind` database comes from an older version of todai: opening it read-write migrates it.
pub fn needs_migration(conn: &Connection, kind: &DatabaseKind) -> Result<bool> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    Ok(version < kind.migrations.len() as i64)
}

/// Writes a copy of `conn`'s database to `to`: consistent even while it's in use, compacted, and self-contained (in
/// rollback journal mode rather than WAL, so without `-wal` and `-shm` files next to it). Its kind and version are
/// kept: it can be opened like the original.
///
/// Written next to `to`, then renamed: `to` is never a partial copy, and is replaced if it exists. `conn` can be
/// read-only, but not within a transaction.
pub fn snapshot(conn: &Connection, to: &Path) -> Result<()> {
    let mut partial = to.as_os_str().to_owned();
    partial.push(".partial");
    let partial = PathBuf::from(partial);
    let partial_db = partial
        .to_str()
        .ok_or_else(|| TodaiError::InvalidPath(format!("{} is not valid UTF-8", to.display())))?;
    // `VACUUM INTO` needs a new file: an interrupted snapshot may have left one.
    remove_file_if_exists(&partial)?;

    let written = (|| -> Result<()> {
        conn.execute("VACUUM INTO ?1", [partial_db])?;
        // Closed right away, before the rename.
        Connection::open_with_flags(
            &partial,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?
        .pragma_update(None, "journal_mode", "DELETE")?;
        fs::rename(&partial, to)?;
        Ok(())
    })();
    if written.is_err() {
        // Best effort: the error worth reporting is the snapshot's.
        let _ = fs::remove_file(&partial);
    }
    written
}

/// Replaces the content of `conn`'s `kind` database with the one of the `kind` database at `from`, left untouched,
/// then migrates it if it comes from an older version of todai. `conn` stays open, and usable afterwards.
///
/// Fails without changing `conn`'s database if `from` is not a `kind` database (blank ones included), or comes from a
/// newer version.
pub fn restore(conn: &mut Connection, from: &Path, kind: &DatabaseKind) -> Result<()> {
    let source = open_read_only(from, kind)?;
    // In WAL mode, a database can only be overwritten by one of the same page size: the rollback journal takes any.
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    let copied = copy_all(&source, conn, from);
    // Back to WAL, whether copied or not: a failed copy leaves the database unchanged.
    configure_and_migrate(conn, kind)?;
    copied?;
    // Its statements were prepared against the previous schema.
    conn.flush_prepared_statement_cache();
    Ok(())
}

/// Copies `source`'s database, read from `path`, over `destination`'s, in a single step.
fn copy_all(source: &Connection, destination: &mut Connection, path: &Path) -> Result<()> {
    match Backup::new(source, destination)?.step(-1)? {
        StepResult::Done => Ok(()),
        // Busy or locked: another connection holds either database, which the app never does.
        step => Err(TodaiError::InvalidDatabase(format!(
            "could not copy {}: {step:?}",
            path.display()
        ))),
    }
}

/// Configures `conn`, checked to be a `kind` database, and migrates it.
fn init(mut conn: Connection, kind: &DatabaseKind) -> Result<Connection> {
    configure_and_migrate(&mut conn, kind)?;
    Ok(conn)
}

fn configure_and_migrate(conn: &mut Connection, kind: &DatabaseKind) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(conn, kind)
}

/// Removes the file at `path`, if any.
pub(crate) fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        removed => removed,
    }
}

/// An in-memory `kind` database, migrated.
#[cfg(test)]
pub fn open_test_database(kind: &DatabaseKind) -> Connection {
    open(":memory:", kind).unwrap()
}

/// A `kind` database at `path`, as created by the first version of todai: to be migrated. Closed once created.
#[cfg(test)]
pub fn create_first_version_test_database(path: &Path, kind: &DatabaseKind) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(kind.migrations[0]).unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
    conn.pragma_update(None, "application_id", kind.application_id)
        .unwrap();
}

/// Accepts a blank database if `allow_blank`, to be initialized as a `kind` one.
fn check_kind(
    conn: &Connection,
    path: &Path,
    kind: &DatabaseKind,
    allow_blank: bool,
) -> Result<()> {
    let invalid = || {
        TodaiError::InvalidDatabase(format!(
            "{} is not a todai {} database",
            path.display(),
            kind.name
        ))
    };
    let header = || -> rusqlite::Result<(i32, i64)> {
        Ok((
            conn.pragma_query_value(None, "application_id", |r| r.get(0))?,
            conn.pragma_query_value(None, "user_version", |r| r.get(0))?,
        ))
    };
    // Reading the header is the first access to the file.
    let (application_id, version) = match header() {
        Err(rusqlite::Error::SqliteFailure(error, _)) if error.code == ErrorCode::NotADatabase => {
            return Err(invalid())
        }
        header => header?,
    };

    let known = match application_id {
        id if id == kind.application_id => true,
        // A new database, unless another application created tables in it.
        0 if version == 0 => {
            let blank = conn.query_row("SELECT COUNT(*) FROM sqlite_schema", [], |r| {
                r.get::<_, i64>(0)
            })? == 0;
            if blank && !allow_blank {
                return Err(TodaiError::InvalidDatabase(format!(
                    "{} is blank, not yet a todai {} database",
                    path.display(),
                    kind.name
                )));
            }
            blank
        }
        _ => false,
    };
    if !known {
        return Err(invalid());
    }
    if version > kind.migrations.len() as i64 {
        return Err(TodaiError::InvalidDatabase(format!(
            "{} comes from a newer version of todai",
            path.display()
        )));
    }
    Ok(())
}

fn migrate(conn: &mut Connection, kind: &DatabaseKind) -> Result<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (i, sql) in kind.migrations.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (i + 1) as i64)?;
        tx.pragma_update(None, "application_id", kind.application_id)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(conn: &Connection) -> (i32, i64) {
        (
            conn.pragma_query_value(None, "application_id", |r| r.get(0))
                .unwrap(),
            conn.pragma_query_value(None, "user_version", |r| r.get(0))
                .unwrap(),
        )
    }

    #[test]
    fn open_migrates_a_new_database_and_marks_its_kind() {
        for kind in [&WORKSPACE, &APP] {
            let conn = open_test_database(kind);
            assert_eq!(
                header(&conn),
                (kind.application_id, kind.migrations.len() as i64)
            );
        }
    }

    #[test]
    fn open_rejects_a_database_of_another_kind() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("workspace.sqlite3");
        let app = dir.path().join("app.sqlite3");
        open(&workspace, &WORKSPACE).unwrap();
        open(&app, &APP).unwrap();
        assert!(matches!(
            open(&workspace, &APP),
            Err(TodaiError::InvalidDatabase(_))
        ));
        assert!(matches!(
            open(&app, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
    }

    #[test]
    fn open_rejects_files_from_other_applications() {
        let dir = tempfile::tempdir().unwrap();

        let text = dir.path().join("notes.txt");
        std::fs::write(
            &text,
            "Not a database, but long enough to have a header.".repeat(10),
        )
        .unwrap();
        assert!(matches!(
            open(&text, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));

        let other = dir.path().join("other.sqlite3");
        Connection::open(&other)
            .unwrap()
            .execute_batch("CREATE TABLE things (id INTEGER PRIMARY KEY)")
            .unwrap();
        assert!(matches!(
            open(&other, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
        // Unchanged.
        assert_eq!(header(&Connection::open(&other).unwrap()), (0, 0));
    }

    #[test]
    fn open_existing_rejects_blank_and_missing_files_without_creating_them() {
        let dir = tempfile::tempdir().unwrap();

        let empty = dir.path().join("empty.sqlite3");
        std::fs::write(&empty, "").unwrap();
        assert!(matches!(
            open_existing(&empty, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
        assert_eq!(std::fs::metadata(&empty).unwrap().len(), 0);

        let blank = dir.path().join("blank.sqlite3");
        // A valid SQLite file, without any table.
        Connection::open(&blank)
            .unwrap()
            .execute_batch("CREATE TABLE things (id INTEGER PRIMARY KEY); DROP TABLE things;")
            .unwrap();
        assert!(std::fs::metadata(&blank).unwrap().len() > 0);
        assert!(matches!(
            open_existing(&blank, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
        assert_eq!(header(&Connection::open(&blank).unwrap()), (0, 0));

        let missing = dir.path().join("missing.sqlite3");
        assert!(open_existing(&missing, &WORKSPACE).is_err());
        assert!(!missing.exists());
    }

    #[test]
    fn open_existing_rejects_other_kinds_and_applications() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app.sqlite3");
        open(&app, &APP).unwrap();
        assert!(matches!(
            open_existing(&app, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));

        let other = dir.path().join("other.sqlite3");
        Connection::open(&other)
            .unwrap()
            .execute_batch("CREATE TABLE things (id INTEGER PRIMARY KEY)")
            .unwrap();
        assert!(matches!(
            open_existing(&other, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
        assert_eq!(header(&Connection::open(&other).unwrap()), (0, 0));
    }

    #[test]
    fn open_existing_migrates_a_database_from_an_older_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite3");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(WORKSPACE.migrations[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
            conn.pragma_update(None, "application_id", WORKSPACE.application_id)
                .unwrap();
        }
        let conn = open_existing(&path, &WORKSPACE).unwrap();
        assert_eq!(
            header(&conn),
            (WORKSPACE.application_id, WORKSPACE.migrations.len() as i64)
        );
    }

    /// A workspace database at `path`, with a tag named `name`.
    fn workspace_with_tag(path: &Path, name: &str) -> Connection {
        let conn = open(path, &WORKSPACE).unwrap();
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES (?1, ?1, 0, 0, 0)",
            [name],
        )
        .unwrap();
        conn
    }

    fn tag_names(conn: &Connection) -> Vec<String> {
        conn.prepare("SELECT name FROM tags ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }

    fn journal_mode(conn: &Connection) -> String {
        conn.pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap()
    }

    fn first_version_workspace(path: &Path) {
        create_first_version_test_database(path, &WORKSPACE);
    }

    #[test]
    fn open_read_only_neither_creates_nor_migrates() {
        let dir = tempfile::tempdir().unwrap();

        let missing = dir.path().join("missing.sqlite3");
        assert!(open_read_only(&missing, &WORKSPACE).is_err());
        assert!(!missing.exists());

        let old = dir.path().join("old.sqlite3");
        first_version_workspace(&old);
        let conn = open_read_only(&old, &WORKSPACE).unwrap();
        assert!(needs_migration(&conn, &WORKSPACE).unwrap());
        assert_eq!(header(&conn), (WORKSPACE.application_id, 1));
        assert!(conn
            .execute_batch("CREATE TABLE things (id INTEGER)")
            .is_err());

        let current = dir.path().join("current.sqlite3");
        open(&current, &WORKSPACE).unwrap();
        let conn = open_read_only(&current, &WORKSPACE).unwrap();
        assert!(!needs_migration(&conn, &WORKSPACE).unwrap());

        let app = dir.path().join("app.sqlite3");
        open(&app, &APP).unwrap();
        assert!(matches!(
            open_read_only(&app, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
    }

    #[test]
    fn snapshot_writes_a_self_contained_copy() {
        let dir = tempfile::tempdir().unwrap();
        let conn = workspace_with_tag(&dir.path().join("work.sqlite3"), "Work");
        assert_eq!(journal_mode(&conn), "wal");

        let copy = dir.path().join("copy.sqlite3");
        snapshot(&conn, &copy).unwrap();
        let copied = open_read_only(&copy, &WORKSPACE).unwrap();
        assert_eq!(
            header(&copied),
            (WORKSPACE.application_id, WORKSPACE.migrations.len() as i64)
        );
        assert_eq!(journal_mode(&copied), "delete");
        assert_eq!(tag_names(&copied), ["Work"]);
        drop(copied);
        for leftover in [
            "copy.sqlite3.partial",
            "copy.sqlite3-wal",
            "copy.sqlite3-shm",
        ] {
            assert!(!dir.path().join(leftover).exists(), "{leftover}");
        }
    }

    #[test]
    fn snapshot_replaces_existing_files_and_reads_closed_databases() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("work.sqlite3");
        // Closed, like the databases of the workspaces that aren't active.
        drop(workspace_with_tag(&path, "Work"));
        let copy = dir.path().join("copy.sqlite3");
        fs::write(&copy, "Previous export").unwrap();
        fs::write(dir.path().join("copy.sqlite3.partial"), "Interrupted").unwrap();

        snapshot(&open_read_only(&path, &WORKSPACE).unwrap(), &copy).unwrap();
        assert_eq!(
            tag_names(&open_read_only(&copy, &WORKSPACE).unwrap()),
            ["Work"]
        );
    }

    #[test]
    fn restore_replaces_the_content_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let backup = dir.path().join("backup.sqlite3");
        snapshot(
            &workspace_with_tag(&dir.path().join("before.sqlite3"), "Before"),
            &backup,
        )
        .unwrap();
        let backup_bytes = fs::read(&backup).unwrap();

        let path = dir.path().join("work.sqlite3");
        let mut conn = workspace_with_tag(&path, "After");
        restore(&mut conn, &backup, &WORKSPACE).unwrap();
        assert_eq!(tag_names(&conn), ["Before"]);
        assert_eq!(journal_mode(&conn), "wal");
        assert!(conn
            .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
            .unwrap());

        // Still usable, and persisted.
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES ('later', 'Later', 0, 0, 0)",
            [],
        )
        .unwrap();
        drop(conn);
        assert_eq!(
            tag_names(&open_existing(&path, &WORKSPACE).unwrap()),
            ["Before", "Later"]
        );
        assert_eq!(fs::read(&backup).unwrap(), backup_bytes);
    }

    #[test]
    fn restore_migrates_a_backup_from_an_older_version() {
        let dir = tempfile::tempdir().unwrap();
        let backup = dir.path().join("backup.sqlite3");
        first_version_workspace(&backup);

        let mut conn = workspace_with_tag(&dir.path().join("work.sqlite3"), "Work");
        restore(&mut conn, &backup, &WORKSPACE).unwrap();
        assert_eq!(
            header(&conn),
            (WORKSPACE.application_id, WORKSPACE.migrations.len() as i64)
        );
        // Created by a later migration.
        assert!(tag_names(&conn).is_empty());
        // The backup itself is left at its version.
        assert_eq!(header(&open_read_only(&backup, &WORKSPACE).unwrap()).1, 1);
    }

    #[test]
    fn restore_accepts_another_page_size() {
        let dir = tempfile::tempdir().unwrap();
        let backup = dir.path().join("backup.sqlite3");
        {
            let conn = Connection::open(&backup).unwrap();
            conn.pragma_update(None, "page_size", 8192).unwrap();
            let conn = init(conn, &WORKSPACE).unwrap();
            conn.execute(
                "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES ('b', 'Backup', 0, 0, 0)",
                [],
            )
            .unwrap();
            assert_eq!(
                conn.pragma_query_value(None, "page_size", |r| r.get::<_, i64>(0))
                    .unwrap(),
                8192
            );
        }

        let mut conn = workspace_with_tag(&dir.path().join("work.sqlite3"), "Work");
        restore(&mut conn, &backup, &WORKSPACE).unwrap();
        assert_eq!(tag_names(&conn), ["Backup"]);
        assert_eq!(journal_mode(&conn), "wal");
    }

    #[test]
    fn restore_refuses_other_databases_without_changes() {
        let dir = tempfile::tempdir().unwrap();
        let mut conn = workspace_with_tag(&dir.path().join("work.sqlite3"), "Work");

        let app = dir.path().join("app.sqlite3");
        open(&app, &APP).unwrap();
        let newer = dir.path().join("newer.sqlite3");
        open(&newer, &WORKSPACE)
            .unwrap()
            .pragma_update(None, "user_version", WORKSPACE.migrations.len() as i64 + 1)
            .unwrap();
        let missing = dir.path().join("missing.sqlite3");
        for path in [&app, &newer, &missing] {
            assert!(restore(&mut conn, path, &WORKSPACE).is_err(), "{path:?}");
        }
        assert!(!missing.exists());
        assert_eq!(tag_names(&conn), ["Work"]);
        assert_eq!(journal_mode(&conn), "wal");
    }

    #[test]
    fn open_adds_backups_to_an_existing_app_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.sqlite3");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(APP.migrations[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
            conn.pragma_update(None, "application_id", APP.application_id)
                .unwrap();
            conn.execute(
                "INSERT INTO workspaces (id, name, path, created_at) VALUES ('w', 'Work', '/work.sqlite3', 0)",
                [],
            )
            .unwrap();
        }

        let conn = open(&path, &APP).unwrap();
        let (auto_backup, auto_backup_keep): (bool, u32) = conn
            .query_row(
                "SELECT auto_backup, auto_backup_keep FROM workspaces WHERE id = 'w'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(auto_backup);
        assert_eq!(auto_backup_keep, 7);
        let backups: i64 = conn
            .query_row("SELECT COUNT(*) FROM workspace_backups", [], |r| r.get(0))
            .unwrap();
        assert_eq!(backups, 0);
    }

    #[test]
    fn open_existing_adds_notes_to_a_workspace_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite3");
        {
            // As left by the last version without notes.
            let conn = Connection::open(&path).unwrap();
            for migration in &WORKSPACE.migrations[..4] {
                conn.execute_batch(migration).unwrap();
            }
            conn.pragma_update(None, "user_version", 4).unwrap();
            conn.pragma_update(None, "application_id", WORKSPACE.application_id)
                .unwrap();
            conn.execute(
                "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES ('w', 'Work', 0, 0, 0)",
                [],
            )
            .unwrap();
        }

        let conn = open_existing(&path, &WORKSPACE).unwrap();
        assert_eq!(header(&conn).1, 5);
        assert_eq!(tag_names(&conn), ["Work"]);
        let notes: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(notes, 0);
    }

    #[test]
    fn open_rejects_a_database_from_a_newer_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite3");
        open(&path, &WORKSPACE)
            .unwrap()
            .pragma_update(None, "user_version", WORKSPACE.migrations.len() as i64 + 1)
            .unwrap();
        assert!(matches!(
            open(&path, &WORKSPACE),
            Err(TodaiError::InvalidDatabase(_))
        ));
    }
}
