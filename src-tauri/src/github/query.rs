//! The GraphQL request fetching several pull requests and issues at once, and its answer.

use std::fmt::Write;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::snapshot::{FetchedItem, RawItem, MAX_CHECKS};
use crate::links::GitHubRef;

/// Pull requests and issues fetched per request: 50 cost 2 of the 5,000 points GitHub allows per hour.
pub const MAX_ITEMS: usize = 50;

/// What's fetched of each pull request or issue: see `snapshot::RawItem`.
const ITEM_FRAGMENT: &str = "fragment item on IssueOrPullRequest {
  __typename
  ... on PullRequest {
    url title state isDraft reviewDecision
    author { login }
    updatedAt additions deletions changedFiles headRefName baseRefName
    statusCheckRollup {
      state
      contexts(first: MAX_CHECKS) {
        checkRunCountsByState { state count }
        statusContextCountsByState { state count }
        nodes {
          __typename
          ... on CheckRun { name status conclusion startedAt checkSuite { workflowRun { workflow { name } } } }
          ... on StatusContext { context state createdAt }
        }
      }
    }
    closingIssuesReferences(first: 5) { nodes { url title state stateReason } }
  }
  ... on Issue {
    url title state stateReason
    author { login }
    updatedAt
    assignees(first: 5) { nodes { login } }
    labels(first: 10) { nodes { name color } }
    milestone { title }
    comments { totalCount }
    closedByPullRequestsReferences(first: 5, includeClosedPrs: true) { nodes { url title state isDraft } }
  }
}";

/// The request's body fetching `refs` (at most `MAX_ITEMS`), each under its own alias, along with the rate limit.
pub fn items_request(refs: &[GitHubRef]) -> Value {
    debug_assert!(refs.len() <= MAX_ITEMS);
    let mut params = String::new();
    let mut fields = String::from("  rateLimit { limit remaining resetAt }\n");
    let mut variables = Map::new();
    for (i, github) in refs.iter().enumerate() {
        let _ = write!(params, "$o{i}: String!, $r{i}: String!, $n{i}: Int!, ");
        let _ = writeln!(
            fields,
            "  {}: repository(owner: $o{i}, name: $r{i}) {{ issueOrPullRequest(number: $n{i}) {{ ...item }} }}",
            alias(i)
        );
        variables.insert(format!("o{i}"), json!(github.owner));
        variables.insert(format!("r{i}"), json!(github.repo));
        variables.insert(format!("n{i}"), json!(github.number));
    }
    let fragment = ITEM_FRAGMENT.replace("MAX_CHECKS", &MAX_CHECKS.to_string());
    json!({
        "query": format!("query Links({}) {{\n{fields}}}\n{fragment}", params.trim_end_matches(", ")),
        "variables": variables,
    })
}

/// The request's body fetching the rate limit only.
pub fn rate_limit_request() -> Value {
    json!({ "query": "query RateLimit { rateLimit { limit remaining resetAt } }" })
}

fn alias(i: usize) -> String {
    format!("l{i}")
}

/// What the GraphQL API answered with HTTP 200: its data, and what it couldn't resolve.
#[derive(Deserialize, Debug, Default)]
pub struct Envelope {
    pub data: Option<Map<String, Value>>,
    #[serde(default)]
    pub errors: Vec<GraphQlError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphQlError {
    /// E.g. `NOT_FOUND`, `FORBIDDEN` or `RATE_LIMITED`.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub message: String,
    /// Where in the data, starting with an alias.
    #[serde(default)]
    pub path: Vec<Value>,
}

impl GraphQlError {
    /// Whether it's about the field `alias`.
    fn concerns(&self, alias: &str) -> bool {
        self.path.first().and_then(Value::as_str) == Some(alias)
    }

    /// Whether it's because the resource's organization requires a SAML single sign-on the token isn't authorized for.
    fn is_sso(&self) -> bool {
        self.message.contains("SAML")
    }
}

/// GitHub's rate limit for an account, in points.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: Timestamp,
}

/// What GitHub answered about a pull request or issue.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemResult {
    Found(Box<FetchedItem>),
    /// Missing, deleted, or private to others: GitHub can't tell these apart.
    NoAccess,
    /// Its organization requires single sign-on, which the token isn't authorized for.
    SsoRequired,
    /// Not what was expected: kept as it was.
    Unreadable,
}

/// The rate limit in `envelope`, if asked for.
pub fn rate_limit(envelope: &Envelope) -> Option<RateLimit> {
    let value = envelope.data.as_ref()?.get("rateLimit")?;
    serde_json::from_value(value.clone()).ok()
}

/// What `envelope` says about each of the `count` items asked for with `items_request`, in the same order.
pub fn items(envelope: &Envelope, count: usize) -> Vec<ItemResult> {
    let data = envelope.data.as_ref();
    (0..count)
        .map(|i| {
            let alias = alias(i);
            let item = data
                .and_then(|data| data.get(&alias))
                .and_then(|repository| repository.get("issueOrPullRequest"))
                .filter(|item| !item.is_null());
            let Some(item) = item else {
                let sso = envelope
                    .errors
                    .iter()
                    .any(|error| error.concerns(&alias) && error.is_sso());
                return if sso {
                    ItemResult::SsoRequired
                } else {
                    ItemResult::NoAccess
                };
            };
            match serde_json::from_value::<RawItem>(item.clone()) {
                Ok(raw) => raw.into_fetched().map_or_else(
                    || {
                        log::warn!("GitHub's {alias} is not on github.com");
                        ItemResult::Unreadable
                    },
                    |fetched| ItemResult::Found(Box::new(fetched)),
                ),
                Err(error) => {
                    log::warn!("Could not read GitHub's {alias}: {error}");
                    ItemResult::Unreadable
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::snapshot::{
        CheckOutcome, ChecksState, GitHubSnapshot, IssueState, IssueStateReason, Label,
        PullRequestState, RelatedState, ReviewDecision,
    };

    fn github(owner: &str, repo: &str, number: u32) -> GitHubRef {
        GitHubRef {
            owner: owner.into(),
            repo: repo.into(),
            number,
        }
    }

    fn fixture() -> Envelope {
        serde_json::from_str(include_str!("../../tests/fixtures/github/items.json")).unwrap()
    }

    fn found(result: &ItemResult) -> &FetchedItem {
        match result {
            ItemResult::Found(item) => item,
            other => panic!("expected an item, got {other:?}"),
        }
    }

    #[test]
    fn items_request_asks_for_each_item_under_its_own_alias() {
        let request = items_request(&[github("cli", "cli", 1), github("o", "r.rs", 2)]);
        let query = request["query"].as_str().unwrap();
        assert!(query.starts_with(
            "query Links($o0: String!, $r0: String!, $n0: Int!, $o1: String!, $r1: String!, $n1: Int!) {"
        ));
        assert!(query.contains("  rateLimit { limit remaining resetAt }\n"));
        assert!(query.contains(
            "  l1: repository(owner: $o1, name: $r1) { issueOrPullRequest(number: $n1) { ...item } }\n"
        ));
        assert!(query.contains("contexts(first: 20)"));
        assert_eq!(
            request["variables"],
            json!({ "o0": "cli", "r0": "cli", "n0": 1, "o1": "o", "r1": "r.rs", "n1": 2 })
        );
    }

    #[test]
    fn items_reads_what_github_answered_about_each_one() {
        let envelope = fixture();
        let results = items(&envelope, 6);

        let open = found(&results[0]);
        assert_eq!(open.link.url, "https://github.com/cli/cli/pull/14640");
        assert_eq!(open.link.label, "cli#14640");
        let GitHubSnapshot::PullRequest(open) = &open.snapshot else {
            panic!("expected a pull request");
        };
        assert_eq!(open.state, PullRequestState::Open);
        assert!(!open.is_draft);
        assert_eq!(open.review_decision, Some(ReviewDecision::ReviewRequired));
        assert_eq!(
            (open.additions, open.deletions, open.changed_files),
            (545, 280, 30)
        );
        assert_eq!(open.author.as_deref(), Some("BagToad"));
        let checks = open.checks.as_ref().unwrap();
        assert_eq!(checks.state, ChecksState::Failure);
        assert_eq!(
            (checks.passed, checks.failed, checks.running, checks.skipped),
            (11, 1, 1, 13)
        );
        let runs: Vec<_> = checks
            .runs
            .iter()
            .map(|run| (run.name.as_str(), run.workflow.as_deref(), run.outcome))
            .collect();
        assert_eq!(
            runs,
            [
                ("govulncheck", Some("Lint"), CheckOutcome::Failed),
                (
                    "CodeQL-Build (go, manual, ./.github/codeql/codeql-config.yml)",
                    Some("Code Scanning"),
                    CheckOutcome::Passed
                ),
                ("label-external", Some("PR Triaging"), CheckOutcome::Skipped),
                ("ci/circleci", None, CheckOutcome::Running),
            ]
        );

        let GitHubSnapshot::PullRequest(merged) = &found(&results[1]).snapshot else {
            panic!("expected a pull request");
        };
        assert_eq!(merged.state, PullRequestState::Merged);
        assert_eq!(merged.review_decision, Some(ReviewDecision::Approved));
        assert_eq!(merged.closes.len(), 1);
        assert_eq!(merged.closes[0].link.label, "cli#14614");
        assert_eq!(merged.closes[0].state, RelatedState::Completed);

        let GitHubSnapshot::Issue(completed) = &found(&results[2]).snapshot else {
            panic!("expected an issue");
        };
        assert_eq!(completed.state, IssueState::Closed);
        assert_eq!(completed.state_reason, Some(IssueStateReason::Completed));
        assert_eq!(completed.assignees, ["babakks", "Copilot"]);
        assert_eq!(
            completed.labels[0],
            Label {
                name: "bug".into(),
                color: "#D73A4A".into()
            }
        );
        assert_eq!(completed.comments, 2);
        assert_eq!(completed.closed_by[0].link.label, "cli#14616");
        assert_eq!(completed.closed_by[0].state, RelatedState::Merged);

        let GitHubSnapshot::Issue(open_issue) = &found(&results[3]).snapshot else {
            panic!("expected an issue");
        };
        assert_eq!(open_issue.state, IssueState::Open);
        assert_eq!(open_issue.state_reason, None);
        assert!(open_issue.closed_by.is_empty());

        assert_eq!(results[4], ItemResult::NoAccess);
        assert_eq!(results[5], ItemResult::SsoRequired);
        assert_eq!(
            rate_limit(&envelope).map(|limit| limit.remaining),
            Some(4985)
        );
    }

    #[test]
    fn items_keeps_the_ones_it_cannot_read_apart() {
        let envelope: Envelope = serde_json::from_value(json!({ "data": {
            "l0": { "issueOrPullRequest": { "__typename": "PullRequest", "url": "https://github.com/o/r/pull/1" } },
            "l1": { "issueOrPullRequest": { "__typename": "Discussion" } },
        }}))
        .unwrap();
        assert_eq!(
            items(&envelope, 3),
            [
                ItemResult::Unreadable,
                ItemResult::Unreadable,
                ItemResult::NoAccess
            ]
        );
    }

    #[test]
    fn snapshots_are_stored_with_their_format() {
        let results = items(&fixture(), 3);
        for result in &results {
            let snapshot = &found(result).snapshot;
            let stored = snapshot.to_db().unwrap();
            assert!(stored.starts_with(r#"{"v":1,"data":{"type":"#));
            assert_eq!(GitHubSnapshot::from_db(&stored).as_ref(), Some(snapshot));
        }
        let other_format = r#"{"v":2,"data":{"type":"issue"}}"#;
        assert_eq!(GitHubSnapshot::from_db(other_format), None);
        assert_eq!(GitHubSnapshot::from_db("not json"), None);
    }

    #[test]
    fn merged_and_closed_items_are_settled() {
        let results = items(&fixture(), 4);
        let settled: Vec<bool> = results
            .iter()
            .map(|result| found(result).snapshot.is_settled())
            .collect();
        assert_eq!(settled, [false, true, true, false]);
    }

    /// Asks github.com through the GitHub CLI, signed in: run with `cargo test -- --ignored`.
    #[tokio::test]
    #[ignore = "depends on the machine's GitHub CLI, signed in, and on github.com"]
    async fn real_github_answers_the_items_request() {
        let gh = crate::gh::GhFinder::default()
            .get(false)
            .await
            .expect("gh is installed");
        let refs = [
            github("cli", "cli", 14616),
            github("cli", "cli", 14614),
            github("cli", "cli", 99_999_999),
        ];
        let mut child = std::process::Command::new(gh.path())
            .args(["api", "graphql", "--input", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        serde_json::to_writer(child.stdin.take().unwrap(), &items_request(&refs)).unwrap();
        // `gh` exits with an error for the missing item, but prints the whole answer.
        let output = child.wait_with_output().unwrap();
        let envelope: Envelope = serde_json::from_slice(&output.stdout).unwrap();

        let results = items(&envelope, refs.len());
        assert!(matches!(
            &found(&results[0]).snapshot,
            GitHubSnapshot::PullRequest(pull) if pull.state == PullRequestState::Merged
        ));
        assert!(matches!(
            &found(&results[1]).snapshot,
            GitHubSnapshot::Issue(_)
        ));
        assert_eq!(results[2], ItemResult::NoAccess);
        println!("{:?}", rate_limit(&envelope));
    }
}
