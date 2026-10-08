use rusqlite::{Connection, ErrorCode, OpenFlags};
use std::path::Path;

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

/// A workspace's todos and tags.
pub const WORKSPACE: DatabaseKind = DatabaseKind {
    name: "workspace",
    // "TDWS" (ToDai WorkSpace).
    application_id: 0x5444_5753,
    migrations: &[
        include_str!("../migrations/001_init.sql"),
        include_str!("../migrations/002_tags.sql"),
        include_str!("../migrations/003_tags_non_unique_name.sql"),
        include_str!("../migrations/004_todo_estimate.sql"),
    ],
};

/// The workspaces, shared by the whole app.
pub const APP: DatabaseKind = DatabaseKind {
    name: "app",
    // "TDAP" (ToDai APp).
    application_id: 0x5444_4150,
    migrations: &[include_str!("../migrations-app/001_init.sql")],
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

/// Configures `conn`, checked to be a `kind` database, and migrates it.
fn init(mut conn: Connection, kind: &DatabaseKind) -> Result<Connection> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&mut conn, kind)?;
    Ok(conn)
}

/// An in-memory `kind` database, migrated.
#[cfg(test)]
pub fn open_test_database(kind: &DatabaseKind) -> Connection {
    open(":memory:", kind).unwrap()
}

/// Accepts a blank database if `allow_blank`, to be initialized as a `kind` one.
fn check_kind(conn: &Connection, path: &Path, kind: &DatabaseKind, allow_blank: bool) -> Result<()> {
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
