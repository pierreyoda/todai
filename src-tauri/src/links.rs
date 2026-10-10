use serde::{Deserialize, Serialize};
use url::Url;

use crate::errors::{Result, TodaiError};

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

/// Why GitHub's last answer had nothing about a link's pull request or issue. Serialized like the `fetch_error` column.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FetchError {
    /// Missing, deleted, or private to others: GitHub can't tell these apart.
    NoAccess,
    /// Its organization requires single sign-on, which the GitHub CLI's token isn't authorized for.
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

/// A pull request or issue on github.com.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRef {
    pub owner: String,
    pub repo: String,
    pub number: u32,
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

    if let Some((kind, github)) = github_ref(&url) {
        return Ok(ParsedLink {
            kind,
            url: github_url(kind, &github),
            label: github_label(&github),
            github: Some(github),
        });
    }
    Ok(ParsedLink {
        kind: LinkKind::Url,
        label: domain_label(host),
        url: url.into(),
        github: None,
    })
}

/// The pull request or issue `url` points to, if any, even on one of its tabs (`/files`, `/commits`...).
fn github_ref(url: &Url) -> Option<(LinkKind, GitHubRef)> {
    if !matches!(url.host_str()?, "github.com" | "www.github.com") {
        return None;
    }
    let mut segments = url.path_segments()?;
    let owner = segments.next().filter(|owner| is_github_owner(owner))?;
    let repo = segments.next().filter(|repo| is_github_repo(repo))?;
    let kind = match segments.next()? {
        "pull" => LinkKind::GithubPull,
        "issues" => LinkKind::GithubIssue,
        _ => return None,
    };
    // Digits only: `parse` would also accept a leading `+`.
    let number = segments
        .next()
        .filter(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))?
        .parse()
        .ok()
        .filter(|number| *number > 0)?;
    Some((
        kind,
        GitHubRef {
            owner: owner.into(),
            repo: repo.into(),
            number,
        },
    ))
}

/// A GitHub account or organization name: letters, digits and hyphens.
fn is_github_owner(owner: &str) -> bool {
    (1..=39).contains(&owner.len())
        && owner
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// A GitHub repository name: letters, digits, `-`, `_` and `.`, but not `.` nor `..`.
fn is_github_repo(repo: &str) -> bool {
    (1..=100).contains(&repo.len())
        && repo != "."
        && repo != ".."
        && repo
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// The normalized URL of a pull request or issue.
pub fn github_url(kind: LinkKind, github: &GitHubRef) -> String {
    let section = if kind == LinkKind::GithubPull {
        "pull"
    } else {
        "issues"
    };
    format!(
        "https://github.com/{}/{}/{section}/{}",
        github.owner, github.repo, github.number
    )
}

/// E.g. `todai#58`: the owner is left out, as it's usually the same across one's links.
pub fn github_label(github: &GitHubRef) -> String {
    format!("{}#{}", github.repo, github.number)
}

/// The domain of a link, without `www.`.
pub fn domain_label(host: &str) -> String {
    host.strip_prefix("www.").unwrap_or(host).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn github(owner: &str, repo: &str, number: u32) -> Option<GitHubRef> {
        Some(GitHubRef {
            owner: owner.into(),
            repo: repo.into(),
            number,
        })
    }

    fn reason(input: &str) -> String {
        match parse(input) {
            Err(TodaiError::InvalidLink(reason)) => reason,
            other => panic!("{input:?} was not rejected: {other:?}"),
        }
    }

    #[test]
    fn recognizes_pull_requests_even_on_their_tabs() {
        for input in [
            "https://github.com/pierreyoda/todai/pull/58",
            "https://github.com/pierreyoda/todai/pull/58/",
            "https://github.com/pierreyoda/todai/pull/58/files",
            "https://github.com/pierreyoda/todai/pull/58/commits?w=1#diff-abc",
            "http://www.github.com/pierreyoda/todai/pull/58",
            "  https://GitHub.com/pierreyoda/todai/pull/58  ",
            "github.com/pierreyoda/todai/pull/58",
        ] {
            assert_eq!(
                parse(input).unwrap(),
                ParsedLink {
                    kind: LinkKind::GithubPull,
                    url: "https://github.com/pierreyoda/todai/pull/58".into(),
                    label: "todai#58".into(),
                    github: github("pierreyoda", "todai", 58),
                },
                "{input:?}"
            );
        }
    }

    #[test]
    fn recognizes_issues() {
        assert_eq!(
            parse("https://github.com/some-org/my_repo.rs/issues/12#issuecomment-1").unwrap(),
            ParsedLink {
                kind: LinkKind::GithubIssue,
                url: "https://github.com/some-org/my_repo.rs/issues/12".into(),
                label: "my_repo.rs#12".into(),
                github: github("some-org", "my_repo.rs", 12),
            }
        );
    }

    #[test]
    fn other_github_pages_are_plain_links() {
        for input in [
            "https://github.com/pierreyoda/todai",
            "https://github.com/pierreyoda/todai/pulls",
            "https://github.com/pierreyoda/todai/issues",
            "https://github.com/pierreyoda/todai/pull/0",
            "https://github.com/pierreyoda/todai/pull/+58",
            "https://github.com/pierreyoda/todai/pull/abc",
            "https://github.com/pierreyoda/todai/pull/99999999999",
            "https://github.com/pierreyoda/todai/discussions/3",
            "https://github.com/pierre_yoda/todai/pull/58",
            "https://github.com/pierreyoda/../pull/58",
            "https://gist.github.com/pierreyoda/todai/pull/58",
        ] {
            let link = parse(input).unwrap();
            assert_eq!(link.kind, LinkKind::Url, "{input:?}");
            assert_eq!(link.github, None, "{input:?}");
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
