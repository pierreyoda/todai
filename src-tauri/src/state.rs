use rusqlite::Connection;
use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, PoisonError},
};

use crate::errors::{Result, TodaiError};

/// When locking both, `app_db` is always locked first.
pub struct AppState {
    /// The app database, listing the workspaces.
    app_db: Mutex<Connection>,
    /// The active workspace's database, if it could be opened.
    db: Mutex<Option<Connection>>,
}

/// Locks `mutex`, recovering from poisoning.
///
/// A panic while holding the lock leaves the connection itself usable (open transactions are rolled back on drop).
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppState {
    pub fn new(app_db: Connection, db: Option<Connection>) -> Self {
        Self {
            app_db: Mutex::new(app_db),
            db: Mutex::new(db),
        }
    }

    /// Locks the app database's connection for the guard's lifetime.
    pub fn app_db(&self) -> MutexGuard<'_, Connection> {
        lock(&self.app_db)
    }

    /// Locks the active workspace's connection for the guard's lifetime.
    ///
    /// Fails if no workspace is open.
    pub fn db(&self) -> Result<DbGuard<'_>> {
        let guard = lock(&self.db);
        if guard.is_none() {
            return Err(TodaiError::NoActiveWorkspace);
        }
        Ok(DbGuard(guard))
    }

    /// Whether a workspace is open.
    pub fn has_db(&self) -> bool {
        lock(&self.db).is_some()
    }

    /// Replaces the active workspace's connection, waiting for the commands using the previous one.
    pub fn replace_db(&self, db: Option<Connection>) {
        let previous = std::mem::replace(&mut *lock(&self.db), db);
        // Closed once unlocked.
        drop(previous);
    }
}

/// The active workspace's connection, locked.
pub struct DbGuard<'a>(MutexGuard<'a, Option<Connection>>);

impl Deref for DbGuard<'_> {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        self.0.as_ref().expect("checked by AppState::db")
    }
}

impl DerefMut for DbGuard<'_> {
    fn deref_mut(&mut self) -> &mut Connection {
        self.0.as_mut().expect("checked by AppState::db")
    }
}
