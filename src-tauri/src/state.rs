use rusqlite::Connection;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, PoisonError},
};

use crate::{
    errors::{Result, TodaiError},
    integrations::{IntegrationProblem, Provider},
};

/// When locking both, `app_db` is always locked first.
pub struct AppState {
    /// The app database, listing the workspaces.
    app_db: Mutex<Connection>,
    /// The active workspace's database, if it could be opened.
    db: Mutex<Option<Connection>>,
    /// Where the workspaces' backups are kept, in the app data directory: created with the first one.
    backups_dir: PathBuf,
    /// By provider and account: why the provider couldn't be asked with it, when it needs the user (see `note`).
    attention: Mutex<HashMap<(Provider, String), IntegrationProblem>>,
    /// Held while refreshing links: one refresh at a time, so that the next one finds the links just fetched.
    refreshing: tokio::sync::Mutex<()>,
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
            attention: Mutex::default(),
            refreshing: tokio::sync::Mutex::default(),
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

    /// Remembers what asking `provider` with its account `account` (e.g. a GitHub CLI account's login) gave: whether it
    /// needs the user (e.g. the GitHub CLI is missing or signed out of it), until asking again succeeds. Other failures
    /// (e.g. offline) change nothing.
    pub fn note<T>(
        &self,
        provider: Provider,
        account: &str,
        result: &std::result::Result<T, IntegrationProblem>,
    ) {
        let mut attention = lock(&self.attention);
        let key = (provider, account.to_owned());
        match result {
            Ok(_) => {
                attention.remove(&key);
            }
            Err(problem) if problem.needs_user() => {
                attention.insert(key, problem.clone());
            }
            Err(_) => {}
        }
    }

    /// Why the account `account` of `provider` needs the user, if it did the last time `provider` was asked with it.
    pub fn attention(&self, provider: Provider, account: &str) -> Option<IntegrationProblem> {
        lock(&self.attention)
            .get(&(provider, account.to_owned()))
            .cloned()
    }

    /// Waits for any other refresh of links to end, then holds others off until the guard is dropped.
    pub async fn refreshing(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.refreshing.lock().await
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
    fn attention_lasts_until_the_provider_is_asked_successfully() {
        let state = AppState::new(open_test_database(&APP), None, PathBuf::from("backups"));
        let github = Provider::GitHub;
        let signed_out = IntegrationProblem::SignedOut {
            login: "octocat".into(),
        };
        assert_eq!(state.attention(github, "octocat"), None);

        state.note::<()>(github, "octocat", &Err(signed_out.clone()));
        assert_eq!(state.attention(github, "octocat"), Some(signed_out.clone()));
        assert_eq!(state.attention(github, "hubot"), None);
        // Not something the user can fix: what was known stays.
        state.note::<()>(github, "octocat", &Err(IntegrationProblem::Offline));
        assert_eq!(state.attention(github, "octocat"), Some(signed_out));
        state.note::<()>(github, "octocat", &Err(IntegrationProblem::GhMissing));
        assert_eq!(
            state.attention(github, "octocat"),
            Some(IntegrationProblem::GhMissing)
        );

        state.note(github, "octocat", &Ok(()));
        assert_eq!(state.attention(github, "octocat"), None);
    }
}
