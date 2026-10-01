use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard, PoisonError};

pub struct AppState {
    db: Mutex<Connection>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self { db: Mutex::new(db) }
    }

    /// Locks the SQLite connection for the guard's lifetime.
    pub fn db(&self) -> MutexGuard<'_, Connection> {
        // A panic while holding the lock leaves the connection itself usable
        // (open transactions are rolled back on drop), so recover from poisoning.
        self.db.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
