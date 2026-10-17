//! What todai keeps of a pull request or issue (see `links::snapshot`), and how it's read from GitHub's answer.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::links::{
    self,
    snapshot::{FetchedItem, LinkSnapshot},
    LinkKind, ParsedLink,
};

/// How many checks are kept, in GitHub's order: the counts cover all of them.
pub(super) const MAX_CHECKS: usize = 20;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestSnapshot {
    pub title: String,
    pub state: PullRequestState,
    pub is_draft: bool,
    /// `None` when its repository requires no review, as most personal ones.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_decision: Option<ReviewDecision>,
    /// `None` without any check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checks: Option<Checks>,
    pub additions: u32,
    pub deletions: u32,
    pub changed_files: u32,
    /// Its branch.
    pub head_ref: String,
    /// The branch it's merged into.
    pub base_ref: String,
    /// `None` for a deleted account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub updated_at: Timestamp,
    /// The first few issues it closes once merged.
    pub closes: Vec<RelatedItem>,
}

impl PullRequestSnapshot {
    /// Whether it's merged or closed.
    pub fn is_settled(&self) -> bool {
        self.state != PullRequestState::Open
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

/// A pull request's checks: GitHub Actions' jobs and other apps' check runs, and commit statuses.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Checks {
    /// All of them, as GitHub sums them up.
    pub state: ChecksState,
    pub passed: u32,
    pub failed: u32,
    pub running: u32,
    /// Skipped, neutral or stale.
    pub skipped: u32,
    /// The first ones, in GitHub's order.
    pub runs: Vec<Check>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChecksState {
    Success,
    Failure,
    Pending,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub name: String,
    /// The GitHub Actions workflow it's a job of, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    pub outcome: CheckOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    Passed,
    Failed,
    Running,
    Skipped,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueSnapshot {
    pub title: String,
    pub state: IssueState,
    /// Why it was closed, if it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_reason: Option<IssueStateReason>,
    /// `None` for a deleted account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub updated_at: Timestamp,
    /// The first few.
    pub assignees: Vec<String>,
    /// The first few.
    pub labels: Vec<Label>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<String>,
    pub comments: u32,
    /// The first few pull requests that close it once merged, or did.
    pub closed_by: Vec<RelatedItem>,
}

impl IssueSnapshot {
    /// Whether it's closed.
    pub fn is_settled(&self) -> bool {
        self.state != IssueState::Open
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStateReason {
    Completed,
    NotPlanned,
    Duplicate,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub name: String,
    /// Format: #RRGGBB.
    pub color: String,
}

/// A pull request or issue mentioned by another one, e.g. an issue a pull request closes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedItem {
    /// As if added as a link.
    #[serde(flatten)]
    pub link: ParsedLink,
    pub title: String,
    pub state: RelatedState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelatedState {
    /// An open pull request or issue.
    Open,
    /// A pull request.
    Draft,
    /// A pull request.
    Merged,
    /// A pull request closed without being merged, or an issue closed for another reason than below.
    Closed,
    /// An issue.
    Completed,
    /// An issue.
    NotPlanned,
    /// An issue.
    Duplicate,
}

// What the GraphQL API answers (see `query::ITEM_FRAGMENT`). Its enums are kept as strings, so that a value added by
// GitHub doesn't make a whole item unreadable.

#[derive(Deserialize, Debug)]
#[serde(tag = "__typename")]
pub(super) enum RawItem {
    PullRequest(RawPullRequest),
    Issue(RawIssue),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawPullRequest {
    url: String,
    title: String,
    state: String,
    is_draft: bool,
    review_decision: Option<String>,
    author: Option<RawActor>,
    updated_at: Timestamp,
    additions: u32,
    deletions: u32,
    changed_files: u32,
    head_ref_name: String,
    base_ref_name: String,
    status_check_rollup: Option<RawRollup>,
    closing_issues_references: Option<RawNodes<RawRelatedIssue>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawIssue {
    url: String,
    title: String,
    state: String,
    state_reason: Option<String>,
    author: Option<RawActor>,
    updated_at: Timestamp,
    assignees: RawNodes<RawActor>,
    labels: Option<RawNodes<RawLabel>>,
    milestone: Option<RawMilestone>,
    comments: RawCount,
    closed_by_pull_requests_references: Option<RawNodes<RawRelatedPullRequest>>,
}

#[derive(Deserialize, Debug)]
struct RawActor {
    login: String,
}

#[derive(Deserialize, Debug)]
struct RawLabel {
    name: String,
    color: String,
}

#[derive(Deserialize, Debug)]
struct RawMilestone {
    title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawCount {
    total_count: u32,
}

/// A connection's nodes, any of which can be `null` in GraphQL.
#[derive(Deserialize, Debug)]
struct RawNodes<T> {
    nodes: Vec<Option<T>>,
}

impl<T> RawNodes<T> {
    fn into_iter(nodes: Option<Self>) -> impl Iterator<Item = T> {
        nodes.into_iter().flat_map(|nodes| nodes.nodes).flatten()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawRollup {
    state: String,
    contexts: RawContexts,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawContexts {
    #[serde(default)]
    check_run_counts_by_state: Vec<RawStateCount>,
    #[serde(default)]
    status_context_counts_by_state: Vec<RawStateCount>,
    nodes: Vec<Option<RawContext>>,
}

#[derive(Deserialize, Debug)]
struct RawStateCount {
    state: String,
    count: u32,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "__typename")]
enum RawContext {
    CheckRun(RawCheckRun),
    StatusContext(RawStatusContext),
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawCheckRun {
    name: String,
    status: String,
    conclusion: Option<String>,
    started_at: Option<Timestamp>,
    check_suite: Option<RawCheckSuite>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawCheckSuite {
    workflow_run: Option<RawWorkflowRun>,
}

#[derive(Deserialize, Debug)]
struct RawWorkflowRun {
    workflow: RawWorkflow,
}

#[derive(Deserialize, Debug)]
struct RawWorkflow {
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawStatusContext {
    context: String,
    state: String,
    created_at: Option<Timestamp>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawRelatedIssue {
    url: String,
    title: String,
    state: String,
    state_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawRelatedPullRequest {
    url: String,
    title: String,
    state: String,
    is_draft: bool,
}

impl RawItem {
    /// `None` if it's not where a pull request or issue is expected to be, on github.com.
    pub(super) fn into_fetched(self) -> Option<FetchedItem> {
        let (url, snapshot) = match self {
            Self::PullRequest(pull) => (pull.url.clone(), LinkSnapshot::PullRequest(pull.into())),
            Self::Issue(issue) => (issue.url.clone(), LinkSnapshot::Issue(issue.into())),
        };
        let link = github_link(&url)?;
        Some(FetchedItem { link, snapshot })
    }
}

/// The pull request or issue at `url`, as a link.
fn github_link(url: &str) -> Option<ParsedLink> {
    links::parse(url)
        .ok()
        .filter(|link| link.kind != LinkKind::Url)
}

impl From<RawPullRequest> for PullRequestSnapshot {
    fn from(pull: RawPullRequest) -> Self {
        Self {
            title: pull.title,
            state: match pull.state.as_str() {
                "MERGED" => PullRequestState::Merged,
                "CLOSED" => PullRequestState::Closed,
                _ => PullRequestState::Open,
            },
            is_draft: pull.is_draft,
            review_decision: pull
                .review_decision
                .as_deref()
                .and_then(|decision| match decision {
                    "APPROVED" => Some(ReviewDecision::Approved),
                    "CHANGES_REQUESTED" => Some(ReviewDecision::ChangesRequested),
                    "REVIEW_REQUIRED" => Some(ReviewDecision::ReviewRequired),
                    _ => None,
                }),
            checks: pull.status_check_rollup.map(Checks::from),
            additions: pull.additions,
            deletions: pull.deletions,
            changed_files: pull.changed_files,
            head_ref: pull.head_ref_name,
            base_ref: pull.base_ref_name,
            author: pull.author.map(|author| author.login),
            updated_at: pull.updated_at,
            closes: RawNodes::into_iter(pull.closing_issues_references)
                .filter_map(|issue| {
                    Some(RelatedItem {
                        link: github_link(&issue.url)?,
                        title: issue.title,
                        state: issue_related_state(&issue.state, issue.state_reason.as_deref()),
                    })
                })
                .collect(),
        }
    }
}

impl From<RawRollup> for Checks {
    fn from(rollup: RawRollup) -> Self {
        let mut checks = Self {
            state: match rollup.state.as_str() {
                "SUCCESS" => ChecksState::Success,
                "FAILURE" | "ERROR" => ChecksState::Failure,
                _ => ChecksState::Pending,
            },
            passed: 0,
            failed: 0,
            running: 0,
            skipped: 0,
            runs: Vec::new(),
        };
        let counts = rollup
            .contexts
            .check_run_counts_by_state
            .iter()
            .map(|count| (check_run_state_outcome(&count.state), count.count));
        let status_counts = rollup
            .contexts
            .status_context_counts_by_state
            .iter()
            .map(|count| (status_state_outcome(&count.state), count.count));
        for (outcome, count) in counts.chain(status_counts) {
            *match outcome {
                CheckOutcome::Passed => &mut checks.passed,
                CheckOutcome::Failed => &mut checks.failed,
                CheckOutcome::Running => &mut checks.running,
                CheckOutcome::Skipped => &mut checks.skipped,
            } += count;
        }
        checks.runs = rollup
            .contexts
            .nodes
            .into_iter()
            .flatten()
            .filter_map(|context| match context {
                RawContext::CheckRun(run) => Some(Check {
                    outcome: match run.status.as_str() {
                        "COMPLETED" => run
                            .conclusion
                            .as_deref()
                            .map_or(CheckOutcome::Skipped, check_run_state_outcome),
                        _ => CheckOutcome::Running,
                    },
                    name: run.name,
                    workflow: run
                        .check_suite
                        .and_then(|suite| suite.workflow_run)
                        .map(|run| run.workflow.name),
                    started_at: run.started_at,
                }),
                RawContext::StatusContext(status) => Some(Check {
                    outcome: status_state_outcome(&status.state),
                    name: status.context,
                    workflow: None,
                    started_at: status.created_at,
                }),
                RawContext::Other => None,
            })
            .take(MAX_CHECKS)
            .collect();
        checks
    }
}

/// A check run's `CheckRunState`, or its `CheckConclusionState` once completed.
fn check_run_state_outcome(state: &str) -> CheckOutcome {
    match state {
        "SUCCESS" => CheckOutcome::Passed,
        "FAILURE" | "TIMED_OUT" | "CANCELLED" | "ACTION_REQUIRED" | "STARTUP_FAILURE" => {
            CheckOutcome::Failed
        }
        "QUEUED" | "IN_PROGRESS" | "WAITING" | "PENDING" | "REQUESTED" => CheckOutcome::Running,
        // `NEUTRAL`, `SKIPPED`, `STALE`, and `COMPLETED` (without a conclusion).
        _ => CheckOutcome::Skipped,
    }
}

/// A commit status' `StatusState`.
fn status_state_outcome(state: &str) -> CheckOutcome {
    match state {
        "SUCCESS" => CheckOutcome::Passed,
        "FAILURE" | "ERROR" => CheckOutcome::Failed,
        _ => CheckOutcome::Running,
    }
}

impl From<RawIssue> for IssueSnapshot {
    fn from(issue: RawIssue) -> Self {
        let state = match issue.state.as_str() {
            "CLOSED" => IssueState::Closed,
            _ => IssueState::Open,
        };
        Self {
            title: issue.title,
            state,
            // Kept as `REOPENED` once reopened.
            state_reason: (state == IssueState::Closed)
                .then(|| issue_state_reason(issue.state_reason.as_deref())),
            author: issue.author.map(|author| author.login),
            updated_at: issue.updated_at,
            assignees: issue
                .assignees
                .nodes
                .into_iter()
                .flatten()
                .map(|assignee| assignee.login)
                .collect(),
            labels: RawNodes::into_iter(issue.labels)
                .map(|label| Label {
                    name: label.name,
                    color: format!("#{}", label.color.to_ascii_uppercase()),
                })
                .collect(),
            milestone: issue.milestone.map(|milestone| milestone.title),
            comments: issue.comments.total_count,
            closed_by: RawNodes::into_iter(issue.closed_by_pull_requests_references)
                .filter_map(|pull| {
                    Some(RelatedItem {
                        link: github_link(&pull.url)?,
                        title: pull.title,
                        state: match pull.state.as_str() {
                            "MERGED" => RelatedState::Merged,
                            "CLOSED" => RelatedState::Closed,
                            _ if pull.is_draft => RelatedState::Draft,
                            _ => RelatedState::Open,
                        },
                    })
                })
                .collect(),
        }
    }
}

/// Why a closed issue was closed: completed if GitHub doesn't tell, as for issues closed before it did.
fn issue_state_reason(reason: Option<&str>) -> IssueStateReason {
    match reason {
        Some("NOT_PLANNED") => IssueStateReason::NotPlanned,
        Some("DUPLICATE") => IssueStateReason::Duplicate,
        _ => IssueStateReason::Completed,
    }
}

fn issue_related_state(state: &str, reason: Option<&str>) -> RelatedState {
    if state != "CLOSED" {
        return RelatedState::Open;
    }
    match issue_state_reason(reason) {
        IssueStateReason::Completed => RelatedState::Completed,
        IssueStateReason::NotPlanned => RelatedState::NotPlanned,
        IssueStateReason::Duplicate => RelatedState::Duplicate,
    }
}
