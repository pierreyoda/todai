//! Backups of each workspace's database: copies kept in the app data directory, listed in the app database.

use jiff::{Timestamp, Zoned};
use rusqlite::{named_params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    database::{
        self,
        models::{DbTimestamp, DbWorkspace, DbWorkspaceBackup},
    },
    errors::{Result, TodaiError},
    workspaces,
};

/// Backups directory name, in the app data directory: one subdirectory per workspace, named after its ID.
pub const BACKUPS_DIR_NAME: &str = "backups";

/// What a backup was made for. Serialized like the `kind` column.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupKind {
    /// Asked for.
    Manual,
    /// Made when the workspace is opened, at most once a day: only the most recent ones are kept.
    Automatic,
    /// Made before migrating the workspace's database to a newer version of todai.
    PreMigration,
    /// The workspace's state before restoring another backup, so that restoring can be undone.
    PreRestore,
}

impl BackupKind {
    const DB_MANUAL: &str = "manual";
    const DB_AUTOMATIC: &str = "automatic";
    const DB_PRE_MIGRATION: &str = "pre_migration";
    const DB_PRE_RESTORE: &str = "pre_restore";

    /// The `kind` column, which also ends the backup's file name.
    pub fn to_db(self) -> &'static str {
        match self {
            Self::Manual => Self::DB_MANUAL,
            Self::Automatic => Self::DB_AUTOMATIC,
            Self::PreMigration => Self::DB_PRE_MIGRATION,
            Self::PreRestore => Self::DB_PRE_RESTORE,
        }
    }
}

/// From the `kind` column.
impl TryFrom<&str> for BackupKind {
    type Error = TodaiError;

    fn try_from(kind: &str) -> std::result::Result<Self, Self::Error> {
        match kind {
            Self::DB_MANUAL => Ok(Self::Manual),
            Self::DB_AUTOMATIC => Ok(Self::Automatic),
            Self::DB_PRE_MIGRATION => Ok(Self::PreMigration),
            Self::DB_PRE_RESTORE => Ok(Self::PreRestore),
            _ => Err(TodaiError::InvalidBackupKind(kind.into())),
        }
    }
}

fn backup_not_found(id: &str) -> TodaiError {
    TodaiError::CommandError(format!("Backup {id} not found"))
}

/// The directory of the workspace `workspace_id`'s backups, in `backups_dir`.
pub fn workspace_dir(backups_dir: &Path, workspace_id: &str) -> PathBuf {
    backups_dir.join(workspace_id)
}

/// `name` for a file name: lowercase letters and digits, each run of other characters becoming one dash.
fn slug(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    match slug.trim_end_matches('-') {
        "" => "workspace".into(),
        slug => slug.into(),
    }
}

/// A path in `dir` for a new `kind` backup of `workspace` made at `now`, e.g.
/// `personal_2026-10-10_14-32-05_manual.sqlite3`: readable when browsing the directory, and sorted by date for a given
/// workspace name. A counter is added if a file already has that name.
fn new_file_path(dir: &Path, workspace: &DbWorkspace, kind: BackupKind, now: &Zoned) -> PathBuf {
    let stem = format!(
        "{}_{}_{}",
        slug(&workspace.name),
        now.strftime("%Y-%m-%d_%H-%M-%S"),
        kind.to_db()
    );
    let mut path = dir.join(format!("{stem}.sqlite3"));
    let mut counter = 2;
    while path.exists() {
        path = dir.join(format!("{stem}_{counter}.sqlite3"));
        counter += 1;
    }
    path
}

fn insert(
    app_db: &Connection,
    workspace_id: &str,
    path: &str,
    kind: BackupKind,
    created_at: DbTimestamp,
) -> Result<DbWorkspaceBackup> {
    Ok(app_db
        .prepare_cached(
            "INSERT INTO workspace_backups (id, workspace_id, path, kind, created_at)
             VALUES (:id, :workspace_id, :path, :kind, :created_at)
             RETURNING *",
        )?
        .query_row(
            named_params! {
                ":id": Uuid::now_v7().to_string(),
                ":workspace_id": workspace_id,
                ":path": path,
                ":kind": kind.to_db(),
                ":created_at": created_at,
            },
            DbWorkspaceBackup::from_row,
        )?)
}

/// Makes a `kind` backup of `workspace`, whose database is `db` (possibly read-only), at `now`: a copy in its directory
/// in `backups_dir`, created if needed, listed in `app_db`.
pub fn create(
    app_db: &Connection,
    backups_dir: &Path,
    workspace: &DbWorkspace,
    db: &Connection,
    kind: BackupKind,
    now: &Zoned,
) -> Result<DbWorkspaceBackup> {
    let dir = workspace_dir(backups_dir, &workspace.id);
    fs::create_dir_all(&dir)?;
    let path = new_file_path(&fs::canonicalize(&dir)?, workspace, kind, now);
    let path_db = workspaces::path_to_db(&path)?;
    database::snapshot(db, &path)?;
    let inserted = insert(
        app_db,
        &workspace.id,
        path_db,
        kind,
        now.timestamp().as_second(),
    );
    if inserted.is_err() {
        // Unlisted, it could never be deleted from the app. Best effort: the error worth reporting is the insertion's.
        let _ = fs::remove_file(&path);
    }
    inserted
}

/// The backups of the workspace `workspace_id`, most recent first.
pub fn list(app_db: &Connection, workspace_id: &str) -> Result<Vec<DbWorkspaceBackup>> {
    let backups = app_db
        .prepare_cached(
            // UUIDv7 IDs follow the order of creation, within a second.
            "SELECT * FROM workspace_backups WHERE workspace_id = ?1 ORDER BY created_at DESC, id DESC",
        )?
        .query_map([workspace_id], DbWorkspaceBackup::from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(backups)
}

/// The backup `id`.
pub fn get(app_db: &Connection, id: &str) -> Result<DbWorkspaceBackup> {
    app_db
        .prepare_cached("SELECT * FROM workspace_backups WHERE id = ?1")?
        .query_row([id], DbWorkspaceBackup::from_row)
        .optional()?
        .ok_or_else(|| backup_not_found(id))
}

/// Deletes `backup`'s file (if it's not already gone), then its row: should the latter fail, it's still listed rather
/// than its file being left behind, unlisted.
fn delete_backup(app_db: &Connection, backup: &DbWorkspaceBackup) -> Result<()> {
    database::remove_file_if_exists(Path::new(&backup.path))?;
    app_db
        .prepare_cached("DELETE FROM workspace_backups WHERE id = ?1")?
        .execute([&backup.id])?;
    Ok(())
}

/// Deletes the backup `id`, with its file.
pub fn delete(app_db: &Connection, id: &str) -> Result<()> {
    delete_backup(app_db, &get(app_db, id)?)
}

/// Deletes all the backups of the workspace `workspace_id`, with their files, then its directory in `backups_dir`, with
/// anything else left in it (e.g. an interrupted backup). Returns how many backups were deleted.
///
/// Should deleting one fail, the ones not deleted yet stay listed.
pub fn delete_all(app_db: &Connection, backups_dir: &Path, workspace_id: &str) -> Result<usize> {
    let backups = list(app_db, workspace_id)?;
    for backup in &backups {
        delete_backup(app_db, backup)?;
    }
    match fs::remove_dir_all(workspace_dir(backups_dir, workspace_id)) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        removed => removed?,
    }
    Ok(backups.len())
}

/// Deletes the automatic backups of the workspace `workspace_id` beyond its `keep` most recent ones, with their file.
/// Returns how many were deleted.
pub fn prune_automatic(app_db: &Connection, workspace_id: &str, keep: u32) -> Result<usize> {
    let pruned = app_db
        .prepare_cached(
            "SELECT * FROM workspace_backups
             WHERE workspace_id = :workspace_id AND kind = :kind
             ORDER BY created_at DESC, id DESC
             LIMIT -1 OFFSET :keep",
        )?
        .query_map(
            named_params! {
                ":workspace_id": workspace_id,
                ":kind": BackupKind::Automatic.to_db(),
                ":keep": keep,
            },
            DbWorkspaceBackup::from_row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for backup in &pruned {
        delete_backup(app_db, backup)?;
    }
    Ok(pruned.len())
}

/// Whether an automatic backup of `workspace` is due at `now`: they're enabled, and its last one, if any, was made on
/// another day in `now`'s time zone.
pub fn is_daily_backup_due(
    app_db: &Connection,
    workspace: &DbWorkspace,
    now: &Zoned,
) -> Result<bool> {
    if !workspace.auto_backup {
        return Ok(false);
    }
    let last: Option<DbTimestamp> = app_db
        .prepare_cached(
            "SELECT MAX(created_at) FROM workspace_backups WHERE workspace_id = ?1 AND kind = ?2",
        )?
        .query_row(
            [workspace.id.as_str(), BackupKind::Automatic.to_db()],
            |row| row.get(0),
        )?;
    let Some(last) = last else {
        return Ok(true);
    };
    let last = Timestamp::from_second(last)?.to_zoned(now.time_zone().clone());
    Ok(last.date() != now.date())
}

/// When the last backup of the workspace `workspace_id` was made, whatever its kind.
pub fn last_created_at(app_db: &Connection, workspace_id: &str) -> Result<Option<DbTimestamp>> {
    Ok(app_db
        .prepare_cached("SELECT MAX(created_at) FROM workspace_backups WHERE workspace_id = ?1")?
        .query_row([workspace_id], |row| row.get(0))?)
}

/// Writes a copy of `db`, a workspace's database, to `to`: unlike its backups, it's not listed.
///
/// `to` must be an absolute file path, in an existing directory. A file there is replaced (as confirmed when picking
/// it), unless it's a workspace's database, or in `backups_dir`, which only the app manages.
pub fn export(app_db: &Connection, backups_dir: &Path, db: &Connection, to: &Path) -> Result<()> {
    workspaces::ensure_absolute(to)?;
    let (Some(dir), Some(file_name)) = (to.parent(), to.file_name()) else {
        return Err(TodaiError::InvalidPath(format!(
            "{} is not a file path",
            to.display()
        )));
    };
    // Canonical, like the paths in the app database.
    let to = fs::canonicalize(dir)?.join(file_name);
    workspaces::ensure_unregistered(app_db, workspaces::path_to_db(&to)?)?;
    ensure_outside(backups_dir, &to)?;
    database::snapshot(db, &to)
}

/// Fails if `path`, absolute, is in `backups_dir`, which only the app manages: e.g. a backup must not become a
/// workspace's database, which deleting the backup would delete.
pub fn ensure_outside(backups_dir: &Path, path: &Path) -> Result<()> {
    workspaces::ensure_absolute(path)?;
    // Nothing to protect yet.
    let Ok(backups_dir) = fs::canonicalize(backups_dir) else {
        return Ok(());
    };
    // Canonical up to its closest existing ancestor: it may not exist yet.
    let mut existing = path;
    let mut missing = Vec::new();
    while !existing.exists() {
        let (Some(parent), Some(name)) = (existing.parent(), existing.file_name()) else {
            break;
        };
        missing.push(name);
        existing = parent;
    }
    let mut canonical = fs::canonicalize(existing)?;
    canonical.extend(missing.iter().rev());
    if canonical.starts_with(&backups_dir) {
        return Err(TodaiError::InvalidPath(format!(
            "{} is in the backups directory, managed by todai",
            path.display()
        )));
    }
    Ok(())
}

/// Fails if `backup` is not one of `workspace`'s backups.
fn ensure_backup_of(workspace: &DbWorkspace, backup: &DbWorkspaceBackup) -> Result<()> {
    if backup.workspace_id != workspace.id {
        return Err(TodaiError::CommandError(format!(
            "Backup {} is not a backup of workspace {:?}",
            backup.id, workspace.name
        )));
    }
    Ok(())
}

/// Replaces the content of `db`, the database of `workspace`, with the one of `backup`, one of its backups, after backing
/// up its current state at `now`: restoring this `pre_restore` backup, returned, undoes it.
///
/// Fails without changing anything if `backup`'s file is missing, isn't a todai workspace database or comes from a
/// newer version.
pub fn restore(
    app_db: &Connection,
    backups_dir: &Path,
    workspace: &DbWorkspace,
    db: &mut Connection,
    backup: &DbWorkspaceBackup,
    now: &Zoned,
) -> Result<DbWorkspaceBackup> {
    ensure_backup_of(workspace, backup)?;
    // Checked first, so that a backup that can't be restored doesn't leave a useless one behind.
    database::open_read_only(&backup.path, &database::WORKSPACE)?;
    let pre_restore = create(
        app_db,
        backups_dir,
        workspace,
        db,
        BackupKind::PreRestore,
        now,
    )?;
    database::restore(db, Path::new(&backup.path), &database::WORKSPACE)?;
    Ok(pre_restore)
}

/// Recreates the missing database of `workspace` at its path from `backup`, one of its backups, and opens it: migrated
/// if the backup comes from an older version of todai.
///
/// Fails if a file is at its path, whose content would be lost, or if its directory is missing (e.g. on a disconnected
/// drive), rather than creating it.
pub fn recreate(workspace: &DbWorkspace, backup: &DbWorkspaceBackup) -> Result<Connection> {
    ensure_backup_of(workspace, backup)?;
    let path = Path::new(&workspace.path);
    if path.exists() {
        return Err(TodaiError::CommandError(format!(
            "A file is already at {}: move it away first, or restore the backup as a new workspace",
            path.display()
        )));
    }
    if !path.parent().is_some_and(Path::is_dir) {
        return Err(TodaiError::CommandError(format!(
            "The directory of {} is missing: reconnect its drive, or restore the backup as a new workspace",
            path.display()
        )));
    }
    let source = database::open_read_only(&backup.path, &database::WORKSPACE)?;
    database::snapshot(&source, path)?;
    database::open_existing(path, &database::WORKSPACE)
}

/// Makes the daily backup of `workspace`, whose database is `db`, if it's due at `now` (see [`is_daily_backup_due`]),
/// then deletes its automatic backups beyond the ones it keeps. Returns the backup, if made.
pub fn back_up_daily(
    app_db: &Connection,
    backups_dir: &Path,
    workspace: &DbWorkspace,
    db: &Connection,
    now: &Zoned,
) -> Result<Option<DbWorkspaceBackup>> {
    if !is_daily_backup_due(app_db, workspace, now)? {
        return Ok(None);
    }
    let backup = create(
        app_db,
        backups_dir,
        workspace,
        db,
        BackupKind::Automatic,
        now,
    )?;
    let pruned = prune_automatic(app_db, &workspace.id, workspace.auto_backup_keep)?;
    log::info!(
        "Backed up workspace {:?} to {} (daily), deleting {pruned} older automatic backups",
        workspace.name,
        backup.path
    );
    Ok(Some(backup))
}

/// Makes a `pre_migration` backup of `workspace` at `now` if its database `source`, opened read-only, comes from an
/// older version of todai: opening it read-write migrates it.
pub fn back_up_before_migration(
    app_db: &Connection,
    backups_dir: &Path,
    workspace: &DbWorkspace,
    source: &Connection,
    now: &Zoned,
) -> Result<()> {
    if database::needs_migration(source, &database::WORKSPACE)? {
        let backup = create(
            app_db,
            backups_dir,
            workspace,
            source,
            BackupKind::PreMigration,
            now,
        )?;
        log::info!(
            "Backed up workspace {:?} to {} before migrating it",
            workspace.name,
            backup.path
        );
    }
    Ok(())
}

/// Opens the database of the registered `workspace`, to use it: backed up first if it comes from an older version of
/// todai, as opening it migrates it; then backed up daily (see [`back_up_daily`]).
///
/// Fails if it can't be opened, or backed up before being migrated: it's never migrated without a copy of the previous
/// version. A failed daily backup is only logged.
pub fn open_workspace(
    app_db: &Connection,
    backups_dir: &Path,
    workspace: &DbWorkspace,
    now: &Zoned,
) -> Result<Connection> {
    // Unchecked when it can't be opened read-only: opening it fails with the reason.
    if let Ok(source) = database::open_read_only(&workspace.path, &database::WORKSPACE) {
        back_up_before_migration(app_db, backups_dir, workspace, &source, now)?;
    }
    let db = workspaces::open_registered(workspace)?;
    if let Err(error) = back_up_daily(app_db, backups_dir, workspace, &db, now) {
        log::warn!(
            "Cannot back up workspace {:?} daily: {error}",
            workspace.name
        );
    }
    Ok(db)
}

#[cfg(test)]
mod tests {
    use jiff::{civil::date, tz};

    use super::*;

    const NOW: DbTimestamp = 1_791_000_000;

    /// `day` of October 2026, at `hour`:`minute`:05 in UTC+02:00 (e.g. Paris).
    fn at(day: i8, hour: i8, minute: i8) -> Zoned {
        date(2026, 10, day)
            .at(hour, minute, 5, 0)
            .to_zoned(tz::TimeZone::fixed(tz::offset(2)))
            .unwrap()
    }

    struct Setup {
        dir: tempfile::TempDir,
        app_db: Connection,
        workspace: DbWorkspace,
        db: Connection,
    }

    impl Setup {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let app_db = database::open_test_database(&database::APP);
            let (workspace, db) = workspaces::create(
                &app_db,
                "Personal Notes",
                &dir.path().join("personal.sqlite3"),
                NOW,
            )
            .unwrap();
            Self {
                dir,
                app_db,
                workspace,
                db,
            }
        }

        fn backups_dir(&self) -> PathBuf {
            self.dir.path().join(BACKUPS_DIR_NAME)
        }

        fn backup(&self, kind: BackupKind, now: &Zoned) -> DbWorkspaceBackup {
            create(
                &self.app_db,
                &self.backups_dir(),
                &self.workspace,
                &self.db,
                kind,
                now,
            )
            .unwrap()
        }

        fn export(&self, to: &Path) -> Result<()> {
            export(&self.app_db, &self.backups_dir(), &self.db, to)
        }

        fn listed(&self) -> Vec<String> {
            list(&self.app_db, &self.workspace.id)
                .unwrap()
                .into_iter()
                .map(|backup| backup.id)
                .collect()
        }

        fn insert_tag(&self, name: &str) {
            self.db
                .execute(
                    "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES (?1, ?1, 0, 0, 0)",
                    [name],
                )
                .unwrap();
        }
    }

    fn tag_names(path: &str) -> Vec<String> {
        database::open_read_only(path, &database::WORKSPACE)
            .unwrap()
            .prepare("SELECT name FROM tags ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }

    #[test]
    fn kinds_round_trip_through_the_database() {
        for kind in [
            BackupKind::Manual,
            BackupKind::Automatic,
            BackupKind::PreMigration,
            BackupKind::PreRestore,
        ] {
            assert_eq!(BackupKind::try_from(kind.to_db()).unwrap(), kind);
        }
        assert!(matches!(
            BackupKind::try_from("weekly"),
            Err(TodaiError::InvalidBackupKind(_))
        ));
    }

    #[test]
    fn slug_keeps_letters_and_digits() {
        assert_eq!(slug("Personal Notes"), "personal-notes");
        assert_eq!(slug("  Été 2026 !"), "été-2026");
        assert_eq!(slug("work/side--project"), "work-side-project");
        assert_eq!(slug("!!!"), "workspace");
    }

    #[test]
    fn create_copies_the_database_into_the_workspace_directory() {
        let setup = Setup::new();
        setup.insert_tag("Work");
        let now = at(10, 14, 32);
        let backup = setup.backup(BackupKind::Manual, &now);

        let expected = fs::canonicalize(setup.backups_dir())
            .unwrap()
            .join(&setup.workspace.id)
            .join("personal-notes_2026-10-10_14-32-05_manual.sqlite3");
        assert_eq!(backup.path, expected.to_str().unwrap());
        assert_eq!(backup.workspace_id, setup.workspace.id);
        assert_eq!(backup.kind, "manual");
        assert_eq!(backup.created_at, now.timestamp().as_second());
        assert_eq!(tag_names(&backup.path), ["Work"]);
        assert_eq!(setup.listed(), [backup.id.as_str()]);
        assert_eq!(get(&setup.app_db, &backup.id).unwrap().path, backup.path);
    }

    #[test]
    fn create_never_overwrites_a_backup() {
        let setup = Setup::new();
        let now = at(10, 14, 32);
        setup.insert_tag("First");
        let first = setup.backup(BackupKind::Manual, &now);
        setup.insert_tag("Second");
        let second = setup.backup(BackupKind::Manual, &now);

        assert!(second.path.ends_with("_manual_2.sqlite3"));
        assert_eq!(tag_names(&first.path), ["First"]);
        assert_eq!(tag_names(&second.path), ["First", "Second"]);
        // Same second: the most recently created first.
        assert_eq!(setup.listed(), [second.id, first.id]);
    }

    #[test]
    fn list_is_most_recent_first() {
        let setup = Setup::new();
        let morning = setup.backup(BackupKind::Automatic, &at(10, 9, 0));
        let evening = setup.backup(BackupKind::Manual, &at(10, 18, 0));
        let noon = setup.backup(BackupKind::PreRestore, &at(10, 12, 0));
        assert_eq!(setup.listed(), [evening.id, noon.id, morning.id]);
    }

    #[test]
    fn delete_removes_the_file_and_the_row() {
        let setup = Setup::new();
        let kept = setup.backup(BackupKind::Manual, &at(10, 9, 0));
        let deleted = setup.backup(BackupKind::Manual, &at(10, 10, 0));

        delete(&setup.app_db, &deleted.id).unwrap();
        assert!(!Path::new(&deleted.path).exists());
        assert!(Path::new(&kept.path).exists());
        assert_eq!(setup.listed(), [kept.id.as_str()]);
        assert!(delete(&setup.app_db, &deleted.id).is_err());

        // Its file already gone, e.g. deleted by hand.
        fs::remove_file(&kept.path).unwrap();
        delete(&setup.app_db, &kept.id).unwrap();
        assert!(setup.listed().is_empty());
    }

    #[test]
    fn prune_keeps_the_most_recent_automatic_backups_only() {
        let setup = Setup::new();
        let pre_restore = setup.backup(BackupKind::PreRestore, &at(1, 8, 0));
        let automatic: Vec<_> = (2..=5)
            .map(|day| setup.backup(BackupKind::Automatic, &at(day, 8, 0)))
            .collect();
        let manual = setup.backup(BackupKind::Manual, &at(3, 12, 0));

        assert_eq!(
            prune_automatic(&setup.app_db, &setup.workspace.id, 2).unwrap(),
            2
        );
        assert_eq!(
            setup.listed(),
            [
                automatic[3].id.clone(),
                automatic[2].id.clone(),
                manual.id,
                pre_restore.id,
            ]
        );
        assert!(!Path::new(&automatic[0].path).exists());
        assert!(!Path::new(&automatic[1].path).exists());
        assert_eq!(
            prune_automatic(&setup.app_db, &setup.workspace.id, 2).unwrap(),
            0
        );
    }

    #[test]
    fn daily_backup_is_due_once_per_local_day() {
        let setup = Setup::new();
        let due = |now: &Zoned| is_daily_backup_due(&setup.app_db, &setup.workspace, now).unwrap();
        assert!(due(&at(10, 8, 0)));

        // Only automatic backups count.
        setup.backup(BackupKind::Manual, &at(10, 8, 0));
        assert!(due(&at(10, 8, 1)));

        setup.backup(BackupKind::Automatic, &at(10, 23, 30));
        assert!(!due(&at(10, 23, 59)));
        // 21:30 then 22:30 in UTC, but another day in UTC+02:00.
        assert!(due(&at(11, 0, 30)));
    }

    #[test]
    fn daily_backup_is_never_due_when_disabled() {
        let setup = Setup::new();
        setup
            .app_db
            .execute("UPDATE workspaces SET auto_backup = 0", [])
            .unwrap();
        let workspace = workspaces::get(&setup.app_db, &setup.workspace.id).unwrap();
        assert!(!workspace.auto_backup);
        assert!(!is_daily_backup_due(&setup.app_db, &workspace, &at(10, 8, 0)).unwrap());
    }

    #[test]
    fn workspaces_have_daily_backups_by_default() {
        let setup = Setup::new();
        assert!(setup.workspace.auto_backup);
        assert_eq!(setup.workspace.auto_backup_keep, 7);
    }

    #[test]
    fn removing_a_workspace_unlists_its_backups_but_keeps_their_files() {
        let setup = Setup::new();
        let backup = setup.backup(BackupKind::Manual, &at(10, 9, 0));
        workspaces::remove(&setup.app_db, &setup.workspace.id).unwrap();
        assert!(setup.listed().is_empty());
        assert!(Path::new(&backup.path).exists());
    }

    #[test]
    fn delete_all_deletes_the_backups_of_a_workspace_and_its_directory() {
        let setup = Setup::new();
        let backups_dir = setup.backups_dir();
        let deleted: Vec<_> = [
            BackupKind::Manual,
            BackupKind::Automatic,
            BackupKind::PreRestore,
        ]
        .into_iter()
        .map(|kind| setup.backup(kind, &at(10, 9, 0)))
        .collect();
        // Left over by an interrupted backup.
        let dir = workspace_dir(&backups_dir, &setup.workspace.id);
        fs::write(dir.join("interrupted.sqlite3.partial"), "").unwrap();
        let (other, other_db) = workspaces::create(
            &setup.app_db,
            "Other",
            &setup.dir.path().join("other.sqlite3"),
            NOW,
        )
        .unwrap();
        let kept = create(
            &setup.app_db,
            &backups_dir,
            &other,
            &other_db,
            BackupKind::Manual,
            &at(10, 9, 0),
        )
        .unwrap();

        assert_eq!(
            delete_all(&setup.app_db, &backups_dir, &setup.workspace.id).unwrap(),
            3
        );
        assert!(setup.listed().is_empty());
        assert!(deleted
            .iter()
            .all(|backup| !Path::new(&backup.path).exists()));
        assert!(!dir.exists());
        assert_eq!(list(&setup.app_db, &other.id).unwrap().len(), 1);
        assert!(Path::new(&kept.path).exists());

        // Nothing left to delete.
        assert_eq!(
            delete_all(&setup.app_db, &backups_dir, &setup.workspace.id).unwrap(),
            0
        );
    }

    #[test]
    fn export_writes_an_unlisted_copy() {
        let setup = Setup::new();
        setup.insert_tag("Work");
        let to = setup.dir.path().join("export.sqlite3");
        fs::write(&to, "Previous export").unwrap();

        setup.export(&to).unwrap();
        assert_eq!(tag_names(to.to_str().unwrap()), ["Work"]);
        assert!(setup.listed().is_empty());
        assert!(setup.export(Path::new("relative.sqlite3")).is_err());
    }

    #[test]
    fn export_refuses_workspace_databases_and_backups() {
        let setup = Setup::new();
        setup.insert_tag("Work");
        let backup = setup.backup(BackupKind::Manual, &at(10, 9, 0));
        // Another workspace's database, through a non-canonical path.
        let (other, _) = workspaces::create(
            &setup.app_db,
            "Other",
            &setup.dir.path().join("other.sqlite3"),
            NOW,
        )
        .unwrap();
        let other_path = setup.dir.path().join("./other.sqlite3");

        for to in [
            Path::new(&setup.workspace.path),
            &other_path,
            Path::new(&backup.path),
        ] {
            assert!(setup.export(to).is_err(), "{to:?}");
        }
        assert!(tag_names(&other.path).is_empty());
        assert_eq!(tag_names(&backup.path), ["Work"]);
    }

    #[test]
    fn last_created_at_is_the_most_recent_backup_of_any_kind() {
        let setup = Setup::new();
        assert_eq!(
            last_created_at(&setup.app_db, &setup.workspace.id).unwrap(),
            None
        );
        setup.backup(BackupKind::Automatic, &at(10, 9, 0));
        let latest = at(10, 12, 0);
        setup.backup(BackupKind::Manual, &latest);
        setup.backup(BackupKind::PreRestore, &at(10, 10, 0));
        assert_eq!(
            last_created_at(&setup.app_db, &setup.workspace.id).unwrap(),
            Some(latest.timestamp().as_second())
        );
    }

    #[test]
    fn ensure_outside_refuses_paths_in_the_backups_directory() {
        let setup = Setup::new();
        let backups_dir = setup.backups_dir();
        let inside = backups_dir.join("new/workspace.sqlite3");
        // Without any backup yet, there's nothing to protect.
        assert!(ensure_outside(&backups_dir, &inside).is_ok());

        let backup = setup.backup(BackupKind::Manual, &at(10, 9, 0));
        for path in [
            inside.as_path(),
            Path::new(&backup.path),
            &backups_dir.join(&setup.workspace.id).join("new.sqlite3"),
            // Through a non-canonical path.
            &setup
                .dir
                .path()
                .join(".")
                .join(BACKUPS_DIR_NAME)
                .join("x.sqlite3"),
        ] {
            assert!(
                matches!(
                    ensure_outside(&backups_dir, path),
                    Err(TodaiError::InvalidPath(_))
                ),
                "{path:?}"
            );
        }
        for path in [
            setup.dir.path().join("outside.sqlite3"),
            setup.dir.path().join("backups-not/x.sqlite3"),
        ] {
            assert!(ensure_outside(&backups_dir, &path).is_ok(), "{path:?}");
        }
    }

    #[test]
    fn restore_replaces_the_content_after_backing_it_up() {
        let mut setup = Setup::new();
        let backups_dir = setup.backups_dir();
        setup.insert_tag("Before");
        let backup = setup.backup(BackupKind::Manual, &at(10, 9, 0));
        setup.insert_tag("After");

        let pre_restore = restore(
            &setup.app_db,
            &backups_dir,
            &setup.workspace,
            &mut setup.db,
            &backup,
            &at(10, 10, 0),
        )
        .unwrap();
        assert_eq!(pre_restore.kind, "pre_restore");
        assert_eq!(tag_names(&setup.workspace.path), ["Before"]);
        assert_eq!(tag_names(&pre_restore.path), ["After", "Before"]);
        assert_eq!(setup.listed(), [pre_restore.id.clone(), backup.id]);

        // Undone by restoring the backup of the state before.
        restore(
            &setup.app_db,
            &backups_dir,
            &setup.workspace,
            &mut setup.db,
            &pre_restore,
            &at(10, 11, 0),
        )
        .unwrap();
        assert_eq!(tag_names(&setup.workspace.path), ["After", "Before"]);
    }

    #[test]
    fn restore_refuses_missing_and_foreign_backups_without_changes() {
        let mut setup = Setup::new();
        let backups_dir = setup.backups_dir();
        setup.insert_tag("Work");
        let missing = setup.backup(BackupKind::Manual, &at(10, 9, 0));
        fs::remove_file(&missing.path).unwrap();
        let (other, other_db) = workspaces::create(
            &setup.app_db,
            "Other",
            &setup.dir.path().join("other.sqlite3"),
            NOW,
        )
        .unwrap();
        let foreign = create(
            &setup.app_db,
            &backups_dir,
            &other,
            &other_db,
            BackupKind::Manual,
            &at(10, 9, 0),
        )
        .unwrap();

        for backup in [&missing, &foreign] {
            assert!(restore(
                &setup.app_db,
                &backups_dir,
                &setup.workspace,
                &mut setup.db,
                backup,
                &at(10, 10, 0),
            )
            .is_err());
        }
        assert_eq!(tag_names(&setup.workspace.path), ["Work"]);
        // Without a useless backup of the state before.
        assert_eq!(setup.listed(), [missing.id]);
    }

    /// A workspace with a tag, and a backup of it: its database closed, as if it wasn't the active one.
    fn closed_workspace_with_backup(setup: &Setup) -> (DbWorkspace, DbWorkspaceBackup) {
        let (workspace, db) = workspaces::create(
            &setup.app_db,
            "Archive",
            &setup.dir.path().join("archive/archive.sqlite3"),
            NOW,
        )
        .unwrap();
        db.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES ('t', 'Archived', 0, 0, 0)",
            [],
        )
        .unwrap();
        let backup = create(
            &setup.app_db,
            &setup.backups_dir(),
            &workspace,
            &db,
            BackupKind::Automatic,
            &at(10, 9, 0),
        )
        .unwrap();
        (workspace, backup)
    }

    #[test]
    fn recreate_rebuilds_a_missing_database() {
        let setup = Setup::new();
        let (workspace, backup) = closed_workspace_with_backup(&setup);
        fs::remove_file(&workspace.path).unwrap();

        let db = recreate(&workspace, &backup).unwrap();
        let name: String = db
            .query_row("SELECT name FROM tags", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Archived");
        let journal_mode: String = db
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        assert_eq!(journal_mode, "wal");
    }

    #[test]
    fn recreate_refuses_existing_files_and_missing_directories() {
        let setup = Setup::new();
        let (workspace, backup) = closed_workspace_with_backup(&setup);
        // Not a database anymore, but maybe worth keeping.
        fs::write(&workspace.path, "Corrupted").unwrap();
        assert!(recreate(&workspace, &backup).is_err());
        assert_eq!(fs::read_to_string(&workspace.path).unwrap(), "Corrupted");

        // E.g. on a disconnected drive.
        let dir = Path::new(&workspace.path).parent().unwrap();
        fs::remove_dir_all(dir).unwrap();
        assert!(recreate(&workspace, &backup).is_err());
        assert!(!dir.exists());

        // Another workspace's backup.
        assert!(recreate(&setup.workspace, &backup).is_err());
    }

    fn kinds(setup: &Setup, workspace: &DbWorkspace) -> Vec<String> {
        list(&setup.app_db, &workspace.id)
            .unwrap()
            .into_iter()
            .map(|backup| backup.kind)
            .collect()
    }

    #[test]
    fn back_up_daily_makes_one_backup_a_day_and_prunes_older_ones() {
        let setup = Setup::new();
        let workspace =
            workspaces::set_auto_backup(&setup.app_db, &setup.workspace.id, true, 2).unwrap();
        let back_up = |now: &Zoned| {
            back_up_daily(
                &setup.app_db,
                &setup.backups_dir(),
                &workspace,
                &setup.db,
                now,
            )
            .unwrap()
        };
        let first = back_up(&at(1, 8, 0)).unwrap();
        assert_eq!(first.kind, "automatic");
        assert!(back_up(&at(1, 20, 0)).is_none());
        setup.backup(BackupKind::Manual, &at(1, 21, 0));
        for day in 2..=4 {
            assert!(back_up(&at(day, 8, 0)).is_some());
        }
        assert_eq!(
            kinds(&setup, &workspace),
            ["automatic", "automatic", "manual"]
        );
        assert!(!Path::new(&first.path).exists());
    }

    #[test]
    fn back_up_daily_does_nothing_when_disabled() {
        let setup = Setup::new();
        let workspace =
            workspaces::set_auto_backup(&setup.app_db, &setup.workspace.id, false, 7).unwrap();
        let backup = back_up_daily(
            &setup.app_db,
            &setup.backups_dir(),
            &workspace,
            &setup.db,
            &at(1, 8, 0),
        )
        .unwrap();
        assert!(backup.is_none());
        assert!(setup.listed().is_empty());
    }

    /// A registered workspace whose database, closed, comes from the first version of todai.
    fn outdated_workspace(setup: &Setup) -> DbWorkspace {
        let (workspace, db) = workspaces::create(
            &setup.app_db,
            "Old",
            &setup.dir.path().join("old.sqlite3"),
            NOW,
        )
        .unwrap();
        drop(db);
        fs::remove_file(&workspace.path).unwrap();
        database::create_first_version_test_database(
            Path::new(&workspace.path),
            &database::WORKSPACE,
        );
        workspace
    }

    fn version(conn: &Connection) -> i64 {
        conn.pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn open_workspace_backs_up_outdated_databases_before_migrating_them() {
        let setup = Setup::new();
        let workspace = outdated_workspace(&setup);

        let db = open_workspace(
            &setup.app_db,
            &setup.backups_dir(),
            &workspace,
            &at(1, 8, 0),
        )
        .unwrap();
        assert!(!database::needs_migration(&db, &database::WORKSPACE).unwrap());
        // Then its daily backup, of the migrated database.
        assert_eq!(kinds(&setup, &workspace), ["automatic", "pre_migration"]);
        let backups = list(&setup.app_db, &workspace.id).unwrap();
        let pre_migration =
            database::open_read_only(&backups[1].path, &database::WORKSPACE).unwrap();
        assert_eq!(version(&pre_migration), 1);
    }

    #[test]
    fn open_workspace_never_migrates_without_a_backup() {
        let setup = Setup::new();
        let workspace = outdated_workspace(&setup);
        // Backups can't be written there.
        let backups_dir = setup.dir.path().join("not-a-directory");
        fs::write(&backups_dir, "").unwrap();

        assert!(open_workspace(&setup.app_db, &backups_dir, &workspace, &at(1, 8, 0)).is_err());
        let db = database::open_read_only(&workspace.path, &database::WORKSPACE).unwrap();
        assert_eq!(version(&db), 1);
    }

    #[test]
    fn open_workspace_opens_even_when_the_daily_backup_fails() {
        let setup = Setup::new();
        let backups_dir = setup.dir.path().join("not-a-directory");
        fs::write(&backups_dir, "").unwrap();
        assert!(
            open_workspace(&setup.app_db, &backups_dir, &setup.workspace, &at(1, 8, 0)).is_ok()
        );
        assert!(setup.listed().is_empty());
    }
}
