use rusqlite::Connection;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, PoisonError},
};

use crate::{
    errors::{Result, TodaiError},
    github::GitHubError,
};

/// When locking both, `app_db` is always locked first.
pub struct AppState {
    /// The app database, listing the workspaces.
    app_db: Mutex<Connection>,
    /// The active workspace's database, if it could be opened.
    db: Mutex<Option<Connection>>,
    /// Where the workspaces' backups are kept, in the app data directory: created with the first one.
    backups_dir: PathBuf,
    /// By GitHub CLI account: why GitHub couldn't be asked with it, when it needs the user (see `note_github`).
    github_attention: Mutex<HashMap<String, GitHubError>>,
}

/// Locks `mutex`, recovering from poisoning.
///
/// A panic while holding the lock leaves the connection itself usable (open transactions are rolled back on drop).
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppState {
    pub fn new(app_db: Connection, db: Option<Connection>, backups_dir: PathBuf) -> Self {
        Self {
            app_db: Mutex::new(app_db),
            db: Mutex::new(db),
            backups_dir,
            github_attention: Mutex::default(),
        }
    }

    pub fn backups_dir(&self) -> &Path {
        &self.backups_dir
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

    /// Remembers what asking GitHub as the GitHub CLI account `login` gave: whether it needs the user, because the GitHub
    /// CLI is missing or signed out of it, until asking again succeeds. Other failures (e.g. offline) change nothing.
    pub fn note_github<T>(&self, login: &str, result: &std::result::Result<T, GitHubError>) {
        let mut attention = lock(&self.github_attention);
        match result {
            Ok(_) => {
                attention.remove(login);
            }
            Err(error @ (GitHubError::SignedOut { .. } | GitHubError::GhMissing)) => {
                attention.insert(login.into(), error.clone());
            }
            Err(_) => {}
        }
    }

    /// Why the GitHub CLI account `login` needs the user, if it did the last time GitHub was asked with it.
    pub fn github_attention(&self, login: &str) -> Option<GitHubError> {
        lock(&self.github_attention).get(login).cloned()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{open_test_database, APP};

    #[test]
    fn github_attention_lasts_until_github_is_asked_successfully() {
        let state = AppState::new(open_test_database(&APP), None, PathBuf::from("backups"));
        let signed_out = GitHubError::SignedOut {
            login: "octocat".into(),
        };
        assert_eq!(state.github_attention("octocat"), None);

        state.note_github::<()>("octocat", &Err(signed_out.clone()));
        assert_eq!(state.github_attention("octocat"), Some(signed_out.clone()));
        assert_eq!(state.github_attention("hubot"), None);
        // Not something the user can fix: what was known stays.
        state.note_github::<()>("octocat", &Err(GitHubError::Offline));
        assert_eq!(state.github_attention("octocat"), Some(signed_out));
        state.note_github::<()>("octocat", &Err(GitHubError::GhMissing));
        assert_eq!(
            state.github_attention("octocat"),
            Some(GitHubError::GhMissing)
        );

        state.note_github("octocat", &Ok(()));
        assert_eq!(state.github_attention("octocat"), None);
    }
}
