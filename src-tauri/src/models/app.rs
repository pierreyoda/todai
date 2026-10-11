use jiff::Timestamp;
use serde::Serialize;

use crate::{database::models::DbWorkspace, errors::TodaiError, github::GitHubError};

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
    /// The GitHub CLI account it uses on github.com, if connected to GitHub.
    pub github_login: Option<String>,
    /// Why its GitHub connection needs the user, if it did the last time GitHub was asked: the GitHub CLI is missing, or
    /// signed out of its account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_attention: Option<GitHubError>,
    /// Whether its links' GitHub statuses are refreshed on their own while todai is open, rather than only when asked.
    pub links_auto_refresh: bool,
    /// Whether a todo whose pull request is merged (or issue completed) offers to be marked as done.
    pub links_suggest_done: bool,
    /// Whether pasting a pull request's or issue's link as a new todo's title offers to create it from the link.
    pub links_paste_to_create: bool,
}

/// Inactive and unavailable, without its last backup nor its GitHub attention: set by [`workspace`].
impl TryFrom<DbWorkspace> for Workspace {
    type Error = TodaiError;

    fn try_from(entry: DbWorkspace) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: entry.id,
            name: entry.name,
            path: entry.path,
            is_active: false,
            available: false,
            created_at: Timestamp::from_second(entry.created_at)?,
            last_opened_at: entry
                .last_opened_at
                .map(Timestamp::from_second)
                .transpose()?,
            last_backup_at: None,
            auto_backup: entry.auto_backup,
            auto_backup_keep: entry.auto_backup_keep,
            github_login: entry.github_login,
            github_attention: None,
            links_auto_refresh: entry.links_auto_refresh,
            links_suggest_done: entry.links_suggest_done,
            links_paste_to_create: entry.links_paste_to_create,
        })
    }
}
