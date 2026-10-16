//! The links on todos: what their URL points to, and what's known of the item there.

pub mod github;
pub mod snapshot;

use serde::{Deserialize, Serialize};
use url::Url;

pub use self::github::GitHubRef;
use crate::{
    errors::{Result, TodaiError},
    integrations::Provider,
};

/// Longest URL accepted, in bytes: longer ones are rarely links worth keeping (e.g. tracking parameters).
pub const MAX_URL_LENGTH: usize = 2048;

/// What a link points to, recognized from its URL. Serialized like the `kind` column.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// A pull request on github.com.
    GithubPull,
    /// An issue on github.com.
    GithubIssue,
    /// Any other web page.
    Url,
}

impl LinkKind {
    const DB_GITHUB_PULL: &str = "github_pull";
    const DB_GITHUB_ISSUE: &str = "github_issue";
    const DB_URL: &str = "url";

    /// The `kind` column.
    pub fn to_db(self) -> &'static str {
        match self {
            Self::GithubPull => Self::DB_GITHUB_PULL,
            Self::GithubIssue => Self::DB_GITHUB_ISSUE,
            Self::Url => Self::DB_URL,
        }
    }

    /// Where the item it points to is fetched from: `None` for any other web page, which has no status.
    pub fn provider(self) -> Option<Provider> {
        match self {
            Self::GithubPull | Self::GithubIssue => Some(Provider::GitHub),
            Self::Url => None,
        }
    }
}

/// From the `kind` column.
impl TryFrom<&str> for LinkKind {
    type Error = TodaiError;

    fn try_from(kind: &str) -> std::result::Result<Self, Self::Error> {
        match kind {
            Self::DB_GITHUB_PULL => Ok(Self::GithubPull),
            Self::DB_GITHUB_ISSUE => Ok(Self::GithubIssue),
            Self::DB_URL => Ok(Self::Url),
            _ => Err(TodaiError::InvalidLink(format!("unknown kind {kind:?}"))),
        }
    }
}

/// Why its provider's last answer had nothing about a link's item. Serialized like the `fetch_error` column.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FetchError {
    /// Missing, deleted, or private to others: GitHub can't tell these apart.
    NoAccess,
    /// GitHub: its organization requires single sign-on, which the GitHub CLI's token isn't authorized for.
    SsoRequired,
}

impl FetchError {
    const DB_NO_ACCESS: &str = "no_access";
    const DB_SSO_REQUIRED: &str = "sso_required";

    /// The `fetch_error` column.
    pub fn to_db(self) -> &'static str {
        match self {
            Self::NoAccess => Self::DB_NO_ACCESS,
            Self::SsoRequired => Self::DB_SSO_REQUIRED,
        }
    }
}

/// From the `fetch_error` column.
impl TryFrom<&str> for FetchError {
    type Error = TodaiError;

    fn try_from(error: &str) -> std::result::Result<Self, Self::Error> {
        match error {
            Self::DB_NO_ACCESS => Ok(Self::NoAccess),
            Self::DB_SSO_REQUIRED => Ok(Self::SsoRequired),
            _ => Err(TodaiError::InvalidLink(format!(
                "unknown fetch error {error:?}"
            ))),
        }
    }
}

/// A URL recognized as a link, ready to be added to a todo.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLink {
    pub kind: LinkKind,
    /// Normalized: a pull request's or issue's URL without its tab, query or fragment; any other URL as serialized
    /// by the URL standard (e.g. with a lowercase host).
    pub url: String,
    /// Short name, as shown on the todo: `repo#number` for GitHub, the domain otherwise.
    pub label: String,
    /// Set exactly for the GitHub kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubRef>,
}

impl ParsedLink {
    /// The item it points to, if it's fetched from a provider.
    pub fn item(&self) -> Option<LinkRef> {
        self.github.clone().map(LinkRef::GitHub)
    }
}

/// The item a link points to, at its provider: what's fetched to tell where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkRef {
    GitHub(GitHubRef),
}

impl LinkRef {
    pub fn provider(&self) -> Provider {
        match self {
            Self::GitHub(_) => Provider::GitHub,
        }
    }

    /// The pull request or issue, for GitHub's.
    pub fn github(&self) -> Option<&GitHubRef> {
        match self {
            Self::GitHub(github) => Some(github),
        }
    }
}

fn invalid(reason: &str) -> TodaiError {
    TodaiError::InvalidLink(reason.into())
}

/// Recognizes what the URL `input` (trimmed) points to, and normalizes it.
///
/// Only http(s) URLs are accepted, so that opening a link never runs anything (`javascript:`, `file:`...). The scheme
/// can be left out, as in some address bars, as long as the domain has a dot: `github.com/owner/repo/pull/1`.
pub fn parse(input: &str) -> Result<ParsedLink> {
    let input = input.trim();
    if input.is_empty() {
        return Err(invalid("it's empty"));
    }
    if input.len() > MAX_URL_LENGTH {
        return Err(TodaiError::InvalidLink(format!(
            "it's longer than {MAX_URL_LENGTH} characters"
        )));
    }
    let url = match Url::parse(input) {
        Ok(url) => url,
        Err(url::ParseError::RelativeUrlWithoutBase) => Url::parse(&format!("https://{input}"))
            .ok()
            .filter(|url| url.host_str().is_some_and(|host| host.contains('.')))
            .ok_or_else(|| invalid("it's not a web address"))?,
        Err(_) => return Err(invalid("it's not a web address")),
    };
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid("only http and https links are supported"));
    }
    let host = url
        .host_str()
        .filter(|host| !host.is_empty())
        .ok_or_else(|| invalid("it has no domain"))?;

    if let Some(link) = github::recognize(&url) {
        return Ok(link);
    }
    Ok(ParsedLink {
        kind: LinkKind::Url,
        label: domain_label(host),
        url: url.into(),
        github: None,
    })
}

/// The domain of a link, without `www.`.
pub fn domain_label(host: &str) -> String {
    host.strip_prefix("www.").unwrap_or(host).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reason(input: &str) -> String {
        match parse(input) {
            Err(TodaiError::InvalidLink(reason)) => reason,
            other => panic!("{input:?} was not rejected: {other:?}"),
        }
    }

    #[test]
    fn plain_links_keep_their_url_and_are_labelled_by_domain() {
        assert_eq!(
            parse("https://www.Docs.rs/keyring/latest/keyring/?search=entry#top").unwrap(),
            ParsedLink {
                kind: LinkKind::Url,
                url: "https://www.docs.rs/keyring/latest/keyring/?search=entry#top".into(),
                label: "docs.rs".into(),
                github: None,
            }
        );
        assert_eq!(
            parse("http://localhost:1420").unwrap().url,
            "http://localhost:1420/"
        );
        assert_eq!(parse("example.org").unwrap().url, "https://example.org/");
    }

    #[test]
    fn rejects_what_is_not_a_web_link() {
        assert_eq!(reason(""), "it's empty");
        assert_eq!(reason("   "), "it's empty");
        assert_eq!(reason("hello"), "it's not a web address");
        assert_eq!(reason("not a link at all"), "it's not a web address");
        assert_eq!(reason("https://"), "it's not a web address");
        for input in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "mailto:someone@example.org",
            "ftp://example.org/file",
            "localhost:1420",
        ] {
            assert_eq!(
                reason(input),
                "only http and https links are supported",
                "{input:?}"
            );
        }
        let long = format!("https://example.org/{}", "a".repeat(MAX_URL_LENGTH));
        assert_eq!(reason(&long), "it's longer than 2048 characters");
    }

    #[test]
    fn link_kind_round_trips_through_the_kind_column() {
        for kind in [LinkKind::GithubPull, LinkKind::GithubIssue, LinkKind::Url] {
            assert_eq!(LinkKind::try_from(kind.to_db()).unwrap(), kind);
            assert_eq!(
                serde_json::to_value(kind).unwrap(),
                serde_json::json!(kind.to_db())
            );
        }
        assert!(LinkKind::try_from("notion_page").is_err());
    }

    #[test]
    fn parsed_link_serializes_in_camel_case_without_absent_github() {
        assert_eq!(
            serde_json::to_value(parse("https://github.com/o/r/issues/1").unwrap()).unwrap(),
            serde_json::json!({
                "kind": "github_issue",
                "url": "https://github.com/o/r/issues/1",
                "label": "r#1",
                "github": { "owner": "o", "repo": "r", "number": 1 },
            })
        );
        assert_eq!(
            serde_json::to_value(parse("https://example.org").unwrap()).unwrap(),
            serde_json::json!({
                "kind": "url",
                "url": "https://example.org/",
                "label": "example.org",
            })
        );
    }
}
