//! What todai keeps of a linked item: plain facts, as its provider tells them. The single status word shown on a link
//! is picked from them by the frontend.

use serde::{Deserialize, Serialize};

use super::ParsedLink;
use crate::github::snapshot::{IssueSnapshot, PullRequestSnapshot};

/// The stored snapshots' format: one in another format is ignored, as if never fetched, until fetched again.
const VERSION: u32 = 1;

/// A linked item, as its provider last told about it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkSnapshot {
    /// On GitHub.
    PullRequest(PullRequestSnapshot),
    /// On GitHub.
    Issue(IssueSnapshot),
}

#[derive(Serialize, Deserialize)]
struct Stored<T> {
    v: u32,
    data: T,
}

impl LinkSnapshot {
    /// For the `snapshot` column.
    pub fn to_db(&self) -> serde_json::Result<String> {
        serde_json::to_string(&Stored {
            v: VERSION,
            data: self,
        })
    }

    /// From the `snapshot` column: `None` if stored in another format.
    pub fn from_db(json: &str) -> Option<Self> {
        match serde_json::from_str::<Stored<Self>>(json) {
            Ok(stored) if stored.v == VERSION => Some(stored.data),
            _ => None,
        }
    }

    /// Whether it's done with (e.g. merged or closed): it then rarely changes.
    pub fn is_settled(&self) -> bool {
        match self {
            Self::PullRequest(pull) => pull.is_settled(),
            Self::Issue(issue) => issue.is_settled(),
        }
    }
}

/// A linked item, with the link it's at.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchedItem {
    /// Where its provider says it is: e.g. a pull request linked as an issue (GitHub redirects one to the other), or in
    /// a renamed repository.
    pub link: ParsedLink,
    pub snapshot: LinkSnapshot,
}

/// What a provider answered about a linked item.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemResult {
    Found(Box<FetchedItem>),
    /// Missing, deleted, or private to others: providers can't tell these apart.
    NoAccess,
    /// GitHub: its organization requires single sign-on, which the token isn't authorized for.
    SsoRequired,
    /// Not what was expected: kept as it was.
    Unreadable,
}
