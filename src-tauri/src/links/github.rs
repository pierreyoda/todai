//! Pull requests and issues on github.com, as recognized from their URL.

use serde::{Deserialize, Serialize};
use url::Url;

use super::{LinkKind, ParsedLink};

/// A pull request or issue on github.com.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRef {
    pub owner: String,
    pub repo: String,
    pub number: u32,
}

/// The pull request or issue `url` points to, if any, even on one of its tabs (`/files`, `/commits`...), as a link.
pub fn recognize(url: &Url) -> Option<ParsedLink> {
    let (kind, github) = github_ref(url)?;
    Some(ParsedLink {
        kind,
        url: self::url(kind, &github),
        label: label(&github),
        github: Some(github),
        notion: None,
    })
}

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
pub fn url(kind: LinkKind, github: &GitHubRef) -> String {
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
pub fn label(github: &GitHubRef) -> String {
    format!("{}#{}", github.repo, github.number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::links::parse;

    fn github(owner: &str, repo: &str, number: u32) -> Option<GitHubRef> {
        Some(GitHubRef {
            owner: owner.into(),
            repo: repo.into(),
            number,
        })
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
                    notion: None,
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
                notion: None,
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
}
