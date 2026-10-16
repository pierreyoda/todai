use jiff::Timestamp;
use serde::Serialize;
use url::Url;

use crate::{
    database::models::DbTodoLink,
    errors::TodaiError,
    links::{self, domain_label, snapshot::LinkSnapshot, FetchError, GitHubRef, LinkKind, LinkRef},
};

/// A link on a todo: a GitHub pull request or issue, or any other web page.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TodoLink {
    pub id: String,
    pub todo_id: String,
    pub kind: LinkKind,
    /// Normalized, see `ParsedLink::url`.
    pub url: String,
    /// See `ParsedLink::label`.
    pub label: String,
    /// Set exactly for the GitHub kinds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubRef>,
    pub created_at: Timestamp,
    /// What its provider last told about its item, if fetched since it was added.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<LinkSnapshot>,
    /// When its provider last answered about it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched_at: Option<Timestamp>,
    /// Why its provider's last answer had nothing about it, if so: `snapshot` is then the one before, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetch_error: Option<FetchError>,
}

impl TodoLink {
    /// The item it points to, if it's fetched from a provider.
    pub fn item(&self) -> Option<LinkRef> {
        self.github.clone().map(LinkRef::GitHub)
    }
}

impl TryFrom<DbTodoLink> for TodoLink {
    type Error = TodaiError;

    fn try_from(link: DbTodoLink) -> std::result::Result<Self, Self::Error> {
        let kind = LinkKind::try_from(link.kind.as_str())?;
        // The columns are either all set, for the GitHub kinds, or all `NULL`.
        let github = match (link.github_owner, link.github_repo, link.github_number) {
            (Some(owner), Some(repo), Some(number)) if kind != LinkKind::Url => Some(GitHubRef {
                owner,
                repo,
                number,
            }),
            (None, None, None) if kind == LinkKind::Url => None,
            _ => {
                return Err(TodaiError::InvalidLink(format!(
                    "unexpected GitHub columns for the {} link {}",
                    link.kind, link.id
                )))
            }
        };
        let label = match &github {
            Some(github) => links::github::label(github),
            // Normalized when added: always parses, but falls back to the URL itself rather than failing the list.
            None => Url::parse(&link.url)
                .ok()
                .and_then(|url| url.host_str().map(domain_label))
                .unwrap_or_else(|| link.url.clone()),
        };
        // Ignored if stored in another format: as if never fetched, until fetched again.
        let snapshot = link.snapshot.as_deref().and_then(LinkSnapshot::from_db);
        Ok(Self {
            id: link.id,
            todo_id: link.todo_id,
            kind,
            url: link.url,
            label,
            github,
            created_at: Timestamp::from_second(link.created_at)?,
            fetched_at: link
                .fetched_at
                .filter(|_| snapshot.is_some() || link.fetch_error.is_some())
                .map(Timestamp::from_second)
                .transpose()?,
            snapshot,
            fetch_error: link
                .fetch_error
                .as_deref()
                .map(FetchError::try_from)
                .transpose()?,
        })
    }
}

/// An account `gh` knows on github.com.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GhAccount {
    pub login: String,
    /// The one `gh` uses by default (`gh auth switch` changes it).
    pub active: bool,
    /// Whether its token works: not once revoked or expired, until signing in again with `gh auth login`.
    pub signed_in: bool,
}

/// What the GitHub CLI offers to connect workspaces with.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GhStatus {
    /// Where `gh` was found, if anywhere.
    pub path: Option<String>,
    /// The accounts it knows on github.com, signed in or not anymore: none without `gh`.
    pub accounts: Vec<GhAccount>,
}

#[cfg(test)]
mod test {
    use crate::models::integrations::{GhAccount, GhStatus};

    #[test]
    fn gh_status_serializes_in_camel_case() {
        let status = GhStatus {
            path: Some("/opt/homebrew/bin/gh".into()),
            accounts: vec![GhAccount {
                login: "pierreyoda".into(),
                active: true,
                signed_in: true,
            }],
        };
        assert_eq!(
            serde_json::to_value(status).unwrap(),
            serde_json::json!({
                "path": "/opt/homebrew/bin/gh",
                "accounts": [{ "login": "pierreyoda", "active": true, "signedIn": true }],
            })
        );
    }
}
