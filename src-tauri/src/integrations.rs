//! What todai's integrations share: the services linked items are fetched from, and why one couldn't be asked.

use jiff::Timestamp;
use serde::Serialize;
use thiserror::Error;

/// A service the pull requests, issues... linked to todos are fetched from. Serialized as the keys of
/// `RefreshOutcome::problems`.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Provider {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "notion")]
    Notion,
}

impl Provider {
    /// Its name, as shown.
    pub fn name(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::Notion => "Notion",
        }
    }
}

/// Why a service couldn't be asked: links then keep showing their last known status.
#[derive(Serialize, Error, Debug, Clone, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum IntegrationProblem {
    /// No connection, or the service failed to answer.
    #[error("unreachable")]
    Offline,
    #[error("The GitHub CLI (gh) was not found")]
    GhMissing,
    /// The GitHub CLI has no working token for the account anymore.
    #[error(
        "The GitHub CLI isn't signed in to {login} anymore: sign in again with `gh auth login`"
    )]
    SignedOut { login: String },
    /// Notion refused the workspace's personal access token: it expired, or was revoked.
    #[error("Notion refused the workspace's token: it expired or was revoked")]
    TokenRefused,
    /// The workspace's Notion token isn't in the Keychain anymore, e.g. deleted from Keychain Access.
    #[error("The workspace's Notion token isn't in the Keychain anymore")]
    TokenMissing,
    /// The service asks to wait until `reset_at`.
    #[error("rate limited until {reset_at}")]
    RateLimited { reset_at: Timestamp },
    #[error("{message}")]
    Failed { message: String },
}

impl IntegrationProblem {
    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed {
            message: message.into(),
        }
    }

    /// Whether only the user can solve it, in todai's settings or elsewhere (e.g. by signing in again), rather than
    /// it passing on its own (e.g. once online again).
    pub fn needs_user(&self) -> bool {
        matches!(
            self,
            Self::SignedOut { .. } | Self::GhMissing | Self::TokenRefused | Self::TokenMissing
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problems_serialize_with_their_kind() {
        assert_eq!(
            serde_json::to_value(IntegrationProblem::Offline).unwrap(),
            serde_json::json!({ "kind": "offline" })
        );
        assert_eq!(
            serde_json::to_value(IntegrationProblem::SignedOut {
                login: "octocat".into()
            })
            .unwrap(),
            serde_json::json!({ "kind": "signed_out", "login": "octocat" })
        );
        assert_eq!(
            serde_json::to_value(IntegrationProblem::TokenRefused).unwrap(),
            serde_json::json!({ "kind": "token_refused" })
        );
        assert_eq!(
            serde_json::to_value(IntegrationProblem::RateLimited {
                reset_at: Timestamp::from_second(1_791_000_000).unwrap()
            })
            .unwrap(),
            serde_json::json!({ "kind": "rate_limited", "resetAt": "2026-10-03T04:00:00Z" })
        );
    }

    #[test]
    fn providers_serialize_by_their_key() {
        assert_eq!(
            serde_json::to_value(Provider::GitHub).unwrap(),
            serde_json::json!("github")
        );
        assert_eq!(
            serde_json::to_value(Provider::Notion).unwrap(),
            serde_json::json!("notion")
        );
        let problems =
            std::collections::BTreeMap::from([(Provider::GitHub, IntegrationProblem::GhMissing)]);
        assert_eq!(
            serde_json::to_value(problems).unwrap(),
            serde_json::json!({ "github": { "kind": "gh_missing" } })
        );
    }

    #[test]
    fn only_some_problems_need_the_user() {
        assert!(IntegrationProblem::GhMissing.needs_user());
        assert!(IntegrationProblem::SignedOut {
            login: "octocat".into()
        }
        .needs_user());
        assert!(IntegrationProblem::TokenRefused.needs_user());
        assert!(IntegrationProblem::TokenMissing.needs_user());
        assert!(!IntegrationProblem::Offline.needs_user());
        assert!(!IntegrationProblem::failed("Nope").needs_user());
    }
}
