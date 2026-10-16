//! What their providers tell about the items linked to todos (GitHub's pull requests and issues), fetched through the
//! active workspace's connection to each, and stored with each link.

use std::collections::{BTreeMap, HashMap};

use jiff::Timestamp;
use rusqlite::{named_params, Connection};
use serde::Serialize;
use tauri::State;

use crate::{
    commands::links::link_not_found,
    database::models::{DbTimestamp, DbTodoLink, DbWorkspace},
    errors::{Result, TodaiError},
    gh::GhFinder,
    github::{GitHub, TokenSource},
    integrations::{IntegrationProblem, Provider},
    links::{
        self,
        snapshot::{ItemResult, LinkSnapshot},
        FetchError, GitHubRef, LinkRef, ParsedLink,
    },
    models::{core::Day, integrations::TodoLink},
    state::AppState,
    workspaces,
};

/// How long an open item's status is shown before being fetched again, in seconds.
const OPEN_TTL: i64 = 5 * 60;

/// The same for a settled one (e.g. a merged or closed pull request), or one linked to a completed todo: it rarely
/// changes anymore.
const SETTLED_TTL: i64 = 24 * 60 * 60;

/// What refreshing links did.
#[derive(Serialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RefreshOutcome {
    /// Links whose status was fetched: the todos are worth listing again if any.
    pub refreshed: u32,
    /// Why a provider couldn't be asked, by provider: its links keep their last known status.
    pub problems: BTreeMap<Provider, IntegrationProblem>,
}

/// The providers linked items are fetched from.
pub(crate) struct Providers<'a, T> {
    pub github: &'a GitHub,
    /// Where GitHub's tokens come from: the GitHub CLI.
    pub gh_tokens: &'a T,
}

/// Refreshes the status of the items linked to the non-deleted todos of `day`, if fetched long enough ago (or whenever
/// if `force`), through the active workspace's connection to their provider. Does nothing for a provider it isn't
/// connected to.
#[tauri::command]
pub async fn refresh_links(
    state: State<'_, AppState>,
    finder: State<'_, GhFinder>,
    github: State<'_, GitHub>,
    day: Day,
    force: Option<bool>,
) -> Result<RefreshOutcome> {
    let providers = Providers {
        github: &github,
        gh_tokens: &*finder,
    };
    let target = Target::Day(day);
    refresh(&state, &providers, &target, force.unwrap_or(false)).await
}

/// Refreshes the status of the item of the link `id`, however recently fetched, as `refresh_links`.
#[tauri::command]
pub async fn refresh_link(
    state: State<'_, AppState>,
    finder: State<'_, GhFinder>,
    github: State<'_, GitHub>,
    id: String,
) -> Result<RefreshOutcome> {
    let providers = Providers {
        github: &github,
        gh_tokens: &*finder,
    };
    refresh(&state, &providers, &Target::Link(id), true).await
}

/// What a link would show once added, before adding it.
#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LinkPreview {
    /// As recognized from its URL, or where its provider says it is: e.g. a pull request pasted as an issue.
    pub link: ParsedLink,
    /// What its provider tells about its item, if it could be asked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<LinkSnapshot>,
    /// Why its provider had nothing to tell about it, if so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetch_error: Option<FetchError>,
    /// Why its provider couldn't be asked, if so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<IntegrationProblem>,
}

/// Recognizes what `url` points to, then, for an item fetched from a provider (e.g. a GitHub pull request or issue),
/// asks it about it through the active workspace's connection, if any. Nothing is stored. Fails if it's not an http(s)
/// URL, as `parse_link`.
#[tauri::command]
pub async fn preview_link(
    state: State<'_, AppState>,
    finder: State<'_, GhFinder>,
    github: State<'_, GitHub>,
    url: String,
) -> Result<LinkPreview> {
    let providers = Providers {
        github: &github,
        gh_tokens: &*finder,
    };
    preview(&state, &providers, &url).await
}

async fn preview(
    state: &AppState,
    providers: &Providers<'_, impl TokenSource>,
    url: &str,
) -> Result<LinkPreview> {
    let mut preview = LinkPreview {
        link: links::parse(url)?,
        snapshot: None,
        fetch_error: None,
        problem: None,
    };
    let Some(item) = preview.link.item() else {
        return Ok(preview);
    };
    let workspace = active_workspace(&state.app_db())?;
    let Some(results) = fetch(state, providers, &workspace, item.provider(), &[item]).await else {
        return Ok(preview);
    };
    match results {
        Ok(results) => match results.into_iter().next() {
            Some(ItemResult::Found(item)) => {
                preview.link = item.link;
                preview.snapshot = Some(item.snapshot);
            }
            Some(ItemResult::NoAccess) => preview.fetch_error = Some(FetchError::NoAccess),
            Some(ItemResult::SsoRequired) => preview.fetch_error = Some(FetchError::SsoRequired),
            Some(ItemResult::Unreadable) | None => {}
        },
        Err(problem) => preview.problem = Some(problem),
    }
    Ok(preview)
}

/// The account through which `workspace` is connected to `provider`, if it is: for Notion, the workspace itself, which
/// has a token of its own.
fn connection(workspace: &DbWorkspace, provider: Provider) -> Option<&str> {
    match provider {
        Provider::GitHub => workspace.github_login.as_deref(),
        Provider::Notion => workspace
            .notion_user_id
            .as_ref()
            .map(|_| workspace.id.as_str()),
    }
}

/// What `provider` says about `items`, all of its own, in the same order, asked through `workspace`'s connection to
/// it: `None` without one.
async fn fetch(
    state: &AppState,
    providers: &Providers<'_, impl TokenSource>,
    workspace: &DbWorkspace,
    provider: Provider,
    items: &[LinkRef],
) -> Option<std::result::Result<Vec<ItemResult>, IntegrationProblem>> {
    let account = connection(workspace, provider)?;
    let results = match provider {
        Provider::GitHub => {
            let refs: Vec<GitHubRef> = items.iter().filter_map(LinkRef::github).cloned().collect();
            providers
                .github
                .fetch_items(providers.gh_tokens, account, &refs)
                .await
        }
        // Notion pages are plain links, until recognized as Notion's: none to fetch yet
        Provider::Notion => return None,
    };
    state.note(provider, account, &results);
    Some(results)
}

/// The links to refresh.
enum Target {
    /// Those of a day's non-deleted todos.
    Day(Day),
    /// One link, of a non-deleted todo.
    Link(String),
}

/// A link whose status is due for a refresh.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DueLink {
    id: String,
    item: LinkRef,
}

async fn refresh(
    state: &AppState,
    providers: &Providers<'_, impl TokenSource>,
    target: &Target,
    force: bool,
) -> Result<RefreshOutcome> {
    let _refreshing = state.refreshing().await;

    // Locked to read the links, then to write what the providers said, but not while waiting for them.
    let (workspace, due) = {
        let app_db = state.app_db();
        let workspace = active_workspace(&app_db)?;
        let db = state.db()?;
        let due = due_links(&db, target, Timestamp::now().as_second(), force)?;
        (workspace, due)
    };

    // By provider, each item once: several todos can link to the same pull request or issue.
    let mut items: BTreeMap<Provider, Vec<LinkRef>> = BTreeMap::new();
    for link in &due {
        let items = items.entry(link.item.provider()).or_default();
        if !items.contains(&link.item) {
            items.push(link.item.clone());
        }
    }
    let mut outcome = RefreshOutcome::default();
    let mut results: HashMap<LinkRef, ItemResult> = HashMap::new();
    for (provider, items) in items {
        match fetch(state, providers, &workspace, provider, &items).await {
            None => {}
            Some(Ok(answered)) => results.extend(items.into_iter().zip(answered)),
            Some(Err(problem)) => {
                log::warn!(
                    "Could not refresh {} items from {}: {problem}",
                    items.len(),
                    provider.name()
                );
                outcome.problems.insert(provider, problem);
            }
        }
    }
    if results.is_empty() {
        return Ok(outcome);
    }

    let app_db = state.app_db();
    let current = active_workspace(&app_db)?;
    let switched = current.id != workspace.id;
    results.retain(|item, _| {
        let provider = item.provider();
        let kept = !switched && connection(&current, provider) == connection(&workspace, provider);
        if !kept {
            log::info!(
                "Dropped the links refreshed from {} for a workspace or account not in use anymore",
                provider.name()
            );
        }
        kept
    });
    if results.is_empty() {
        return Ok(outcome);
    }
    let mut db = state.db()?;
    outcome.refreshed = save_results(&mut db, &due, &results, Timestamp::now().as_second())?;
    Ok(outcome)
}

fn active_workspace(app_db: &Connection) -> Result<DbWorkspace> {
    let id = workspaces::active_id(app_db)?.ok_or(TodaiError::NoActiveWorkspace)?;
    workspaces::get(app_db, &id)
}

/// The links of `target` fetched from a provider whose status is due for a refresh at `now`, in the order they were
/// added.
fn due_links(
    db: &Connection,
    target: &Target,
    now: DbTimestamp,
    force: bool,
) -> Result<Vec<DueLink>> {
    const SELECT: &str = "SELECT todo_links.*, todos.completed AS todo_completed
         FROM todos
         JOIN todo_links ON todo_links.todo_id = todos.id";
    let (filter, param) = match target {
        Target::Day(day) => (
            "todos.day = ?1 AND todo_links.kind <> 'url'",
            day.raw.as_str(),
        ),
        Target::Link(id) => ("todo_links.id = ?1", id.as_str()),
    };
    let mut statement = db.prepare_cached(&format!(
        "{SELECT} WHERE {filter} AND todos.deleted_at IS NULL
         ORDER BY todo_links.created_at, todo_links.id"
    ))?;
    let rows = statement.query_map([param], |row| {
        Ok((
            DbTodoLink::from_row(row)?,
            row.get::<_, bool>("todo_completed")?,
        ))
    })?;
    let mut due = Vec::new();
    for row in rows {
        let (link, todo_completed) = row?;
        let link = TodoLink::try_from(link)?;
        if let Target::Link(id) = target {
            if link.kind.provider().is_none() {
                return Err(TodaiError::CommandError(format!(
                    "Link {id} has no status to refresh: it's a plain web link"
                )));
            }
        }
        if let Some(item) = link
            .item()
            .filter(|_| force || is_due(&link, todo_completed, now))
        {
            due.push(DueLink { id: link.id, item });
        }
    }
    if let (Target::Link(id), true) = (target, due.is_empty()) {
        return Err(link_not_found(id));
    }
    Ok(due)
}

/// Whether the status of `link`, of a todo `todo_completed` or not, is old enough to be fetched again at `now`.
fn is_due(link: &TodoLink, todo_completed: bool, now: DbTimestamp) -> bool {
    let Some(fetched_at) = link.fetched_at else {
        return true;
    };
    let settled = todo_completed || link.snapshot.as_ref().is_some_and(LinkSnapshot::is_settled);
    let ttl = if settled { SETTLED_TTL } else { OPEN_TTL };
    now - fetched_at.as_second() >= ttl
}

/// Stores what their provider said about each of the `due` links, at `now`, returning how many were updated.
fn save_results(
    db: &mut Connection,
    due: &[DueLink],
    results: &HashMap<LinkRef, ItemResult>,
    now: DbTimestamp,
) -> Result<u32> {
    let tx = db.transaction()?;
    let mut refreshed = 0;
    for link in due {
        let error = match results.get(&link.item) {
            Some(ItemResult::Found(item)) => {
                // Where its provider says it is (e.g. a pull request linked as an issue, or in a renamed repository),
                // unless its todo already links there too.
                if let Some(github) = &item.link.github {
                    tx.prepare_cached(
                        "UPDATE OR IGNORE todo_links
                         SET kind = :kind, url = :url, github_owner = :owner, github_repo = :repo, github_number = :number
                         WHERE id = :id AND url <> :url",
                    )?
                    .execute(named_params! {
                        ":id": link.id,
                        ":kind": item.link.kind.to_db(),
                        ":url": item.link.url,
                        ":owner": github.owner,
                        ":repo": github.repo,
                        ":number": github.number,
                    })?;
                }
                let updated = tx
                    .prepare_cached(
                        "UPDATE todo_links SET snapshot = :snapshot, fetched_at = :now, fetch_error = NULL
                         WHERE id = :id",
                    )?
                    .execute(named_params! {
                        ":id": link.id,
                        ":snapshot": item.snapshot.to_db()?,
                        ":now": now,
                    })?;
                refreshed += updated as u32;
                continue;
            }
            Some(ItemResult::NoAccess) => FetchError::NoAccess,
            Some(ItemResult::SsoRequired) => FetchError::SsoRequired,
            // Kept as it was, as are the links of providers that couldn't be asked.
            Some(ItemResult::Unreadable) | None => continue,
        };
        // Its last known status, if any, is kept.
        let updated = tx
            .prepare_cached(
                "UPDATE todo_links SET fetched_at = :now, fetch_error = :error WHERE id = :id",
            )?
            .execute(named_params! { ":id": link.id, ":now": now, ":error": error.to_db() })?;
        refreshed += updated as u32;
    }
    tx.commit()?;
    Ok(refreshed)
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, OptionalExtension};
    use serde_json::{json, Value};

    use super::*;
    use crate::{
        database,
        github::{
            snapshot::{IssueSnapshot, IssueState, IssueStateReason, PullRequestState},
            testing::FakeTokens,
        },
        links::LinkKind,
        models::core::Day,
        testing::{ok, FakeServer},
    };

    const NOW: i64 = 1_791_000_000;

    fn providers<'a, T>(github: &'a GitHub, gh_tokens: &'a T) -> Providers<'a, T> {
        Providers { github, gh_tokens }
    }

    fn github_problem(problem: IntegrationProblem) -> BTreeMap<Provider, IntegrationProblem> {
        BTreeMap::from([(Provider::GitHub, problem)])
    }

    struct Setup {
        _dir: tempfile::TempDir,
        state: AppState,
        other: DbWorkspace,
    }

    /// The active workspace, connected to GitHub as `octocat` if `connected`, with the todo `t` on 2026-10-10; and
    /// another workspace.
    fn setup(connected: bool) -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let app_db = database::open_test_database(&database::APP);
        let (active, db) =
            workspaces::create(&app_db, "Active", &dir.path().join("active.sqlite3"), NOW).unwrap();
        let (other, _) =
            workspaces::create(&app_db, "Other", &dir.path().join("other.sqlite3"), NOW).unwrap();
        workspaces::set_active(&app_db, &active.id, NOW).unwrap();
        if connected {
            workspaces::set_github_login(&app_db, &active.id, Some("octocat")).unwrap();
        }
        insert_todo(&db, "t", "2026-10-10");
        let backups_dir = dir.path().join("backups");
        Setup {
            _dir: dir,
            state: AppState::new(app_db, Some(db), backups_dir),
            other,
        }
    }

    fn insert_todo(db: &Connection, id: &str, day: &str) {
        db.execute(
            "INSERT INTO todos (id, day, title, position, created_at, updated_at) VALUES (?1, ?2, ?1, 'a0', 0, 0)",
            (id, day),
        )
        .unwrap();
    }

    /// Adds the link `url` as `id` to the todo `todo_id`, last fetched at `fetched_at` with `snapshot`.
    fn add_link(
        state: &AppState,
        id: &str,
        todo_id: &str,
        url: &str,
        fetched: Option<(DbTimestamp, &LinkSnapshot)>,
    ) {
        let link = links::parse(url).unwrap();
        let github = link.github.as_ref();
        state
            .db()
            .unwrap()
            .execute(
                "INSERT INTO todo_links
                 (id, todo_id, kind, url, github_owner, github_repo, github_number, created_at, snapshot, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
                params![
                    id,
                    todo_id,
                    link.kind.to_db(),
                    link.url,
                    github.map(|github| &github.owner),
                    github.map(|github| &github.repo),
                    github.map(|github| github.number),
                    fetched.map(|(_, snapshot)| snapshot.to_db().unwrap()),
                    fetched.map(|(at, _)| at),
                ],
            )
            .unwrap();
    }

    fn stored(state: &AppState, id: &str) -> TodoLink {
        state
            .db()
            .unwrap()
            .query_row(
                "SELECT * FROM todo_links WHERE id = ?1",
                [id],
                DbTodoLink::from_row,
            )
            .optional()
            .unwrap()
            .map(|link| TodoLink::try_from(link).unwrap())
            .expect("link exists")
    }

    fn issue(state: IssueState) -> LinkSnapshot {
        LinkSnapshot::Issue(IssueSnapshot {
            title: "Issue".into(),
            state,
            state_reason: (state == IssueState::Closed).then_some(IssueStateReason::Completed),
            author: None,
            updated_at: Timestamp::from_second(NOW).unwrap(),
            assignees: Vec::new(),
            labels: Vec::new(),
            milestone: None,
            comments: 0,
            closed_by: Vec::new(),
        })
    }

    fn day() -> Day {
        "2026-10-10".to_string().try_into().unwrap()
    }

    fn due_ids(
        state: &AppState,
        target: &Target,
        now: DbTimestamp,
        force: bool,
    ) -> Result<Vec<String>> {
        Ok(due_links(&*state.db()?, target, now, force)?
            .into_iter()
            .map(|link| link.id)
            .collect())
    }

    /// What GitHub answers about the items asked for, in order (`null` when it can't see one).
    fn answer(items: &[Value]) -> String {
        let mut data = serde_json::Map::new();
        data.insert(
            "rateLimit".into(),
            json!({ "limit": 5000, "remaining": 4999, "resetAt": "2026-10-10T21:00:00Z" }),
        );
        for (i, item) in items.iter().enumerate() {
            data.insert(format!("l{i}"), json!({ "issueOrPullRequest": item }));
        }
        json!({ "data": data }).to_string()
    }

    fn raw_issue(url: &str, state: &str) -> Value {
        json!({
            "__typename": "Issue", "url": url, "title": "Issue", "state": state, "stateReason": null,
            "updatedAt": "2026-10-10T10:00:00Z", "assignees": { "nodes": [] }, "comments": { "totalCount": 0 },
        })
    }

    fn raw_pull(url: &str, state: &str) -> Value {
        json!({
            "__typename": "PullRequest", "url": url, "title": "Pull request", "state": state, "isDraft": false,
            "updatedAt": "2026-10-10T10:00:00Z", "additions": 1, "deletions": 0, "changedFiles": 1,
            "headRefName": "feature", "baseRefName": "main",
        })
    }

    #[test]
    fn links_are_due_once_their_status_is_old_enough() {
        let s = setup(true);
        let (open, closed) = (issue(IssueState::Open), issue(IssueState::Closed));
        let url = |n: u32| format!("https://github.com/o/r/issues/{n}");
        add_link(&s.state, "never", "t", &url(1), None);
        add_link(
            &s.state,
            "open-fresh",
            "t",
            &url(2),
            Some((NOW - OPEN_TTL + 1, &open)),
        );
        add_link(
            &s.state,
            "open-old",
            "t",
            &url(3),
            Some((NOW - OPEN_TTL, &open)),
        );
        add_link(
            &s.state,
            "closed-fresh",
            "t",
            &url(4),
            Some((NOW - OPEN_TTL, &closed)),
        );
        add_link(
            &s.state,
            "closed-old",
            "t",
            &url(5),
            Some((NOW - SETTLED_TTL, &closed)),
        );
        add_link(&s.state, "web", "t", "https://example.org", None);

        let today = Target::Day(day());
        assert_eq!(
            due_ids(&s.state, &today, NOW, false).unwrap(),
            ["closed-old", "never", "open-old"]
        );
        // Except the plain web link, which has no status.
        assert_eq!(due_ids(&s.state, &today, NOW, true).unwrap().len(), 5);
    }

    #[test]
    fn links_of_completed_todos_are_due_daily() {
        let s = setup(true);
        insert_todo(&s.state.db().unwrap(), "done", "2026-10-10");
        s.state
            .db()
            .unwrap()
            .execute("UPDATE todos SET completed = 1 WHERE id = 'done'", [])
            .unwrap();
        let open = issue(IssueState::Open);
        add_link(
            &s.state,
            "a",
            "done",
            "https://github.com/o/r/issues/1",
            Some((NOW - OPEN_TTL, &open)),
        );
        add_link(
            &s.state,
            "b",
            "done",
            "https://github.com/o/r/issues/2",
            Some((NOW - SETTLED_TTL, &open)),
        );
        assert_eq!(
            due_ids(&s.state, &Target::Day(day()), NOW, false).unwrap(),
            ["b"]
        );
    }

    #[test]
    fn links_of_deleted_todos_and_other_days_are_left_out() {
        let s = setup(true);
        {
            let db = s.state.db().unwrap();
            insert_todo(&db, "deleted", "2026-10-10");
            db.execute("UPDATE todos SET deleted_at = 1 WHERE id = 'deleted'", [])
                .unwrap();
            insert_todo(&db, "tomorrow", "2026-10-11");
        }
        add_link(&s.state, "a", "t", "https://github.com/o/r/pull/1", None);
        add_link(
            &s.state,
            "b",
            "deleted",
            "https://github.com/o/r/pull/2",
            None,
        );
        add_link(
            &s.state,
            "c",
            "tomorrow",
            "https://github.com/o/r/pull/3",
            None,
        );
        assert_eq!(
            due_ids(&s.state, &Target::Day(day()), NOW, false).unwrap(),
            ["a"]
        );

        // Nor can they be refreshed one by one.
        let one = |id: &str| due_ids(&s.state, &Target::Link(id.into()), NOW, true);
        assert_eq!(one("a").unwrap(), ["a"]);
        assert!(one("b").is_err());
        assert!(one("unknown").is_err());
        add_link(&s.state, "web", "t", "https://example.org", None);
        assert!(one("web").is_err());
    }

    #[tokio::test]
    async fn refresh_stores_what_github_said_about_the_due_links() {
        let s = setup(true);
        insert_todo(&s.state.db().unwrap(), "u", "2026-10-10");
        add_link(&s.state, "a", "t", "https://github.com/o/r/issues/1", None);
        add_link(&s.state, "b", "t", "https://github.com/o/r/pull/2", None);
        add_link(&s.state, "c", "t", "https://github.com/o/r/issues/3", None);
        // The same issue as `a`, on another todo: asked for once.
        add_link(&s.state, "d", "u", "https://github.com/o/r/issues/1", None);
        let server = FakeServer::serve(vec![ok(answer(&[
            raw_issue("https://github.com/o/r/issues/1", "OPEN"),
            raw_pull("https://github.com/o/r/pull/2", "MERGED"),
            Value::Null,
        ]))]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();

        let outcome = refresh(
            &s.state,
            &providers(&github, &tokens),
            &Target::Day(day()),
            false,
        )
        .await
        .unwrap();
        assert_eq!(
            outcome,
            RefreshOutcome {
                refreshed: 4,
                problems: BTreeMap::new()
            }
        );
        for id in ["a", "d"] {
            let link = stored(&s.state, id);
            assert!(
                matches!(&link.snapshot, Some(LinkSnapshot::Issue(issue)) if issue.state == IssueState::Open)
            );
            assert!(link.fetched_at.is_some());
            assert_eq!(link.fetch_error, None);
        }
        assert!(matches!(
            stored(&s.state, "b").snapshot,
            Some(LinkSnapshot::PullRequest(pull)) if pull.state == PullRequestState::Merged
        ));
        let no_access = stored(&s.state, "c");
        assert_eq!(no_access.fetch_error, Some(FetchError::NoAccess));
        assert_eq!(no_access.snapshot, None);
        let json = serde_json::to_value(stored(&s.state, "b")).unwrap();
        assert_eq!(json["snapshot"]["type"], "pull_request");
        assert_eq!(json["snapshot"]["state"], "merged");
        assert!(json["fetchedAt"].is_string());

        // Nothing is due anymore: GitHub isn't asked again.
        let outcome = refresh(
            &s.state,
            &providers(&github, &tokens),
            &Target::Day(day()),
            false,
        )
        .await
        .unwrap();
        assert_eq!(outcome, RefreshOutcome::default());
        assert_eq!(server.authorizations(), ["Bearer t1"]);
    }

    #[tokio::test]
    async fn refresh_follows_items_to_where_github_says_they_are() {
        let s = setup(true);
        // A pull request linked as an issue.
        add_link(&s.state, "a", "t", "https://github.com/o/r/issues/7", None);
        // In a renamed repository, whose new name the todo also links to.
        add_link(&s.state, "b", "t", "https://github.com/o/old/pull/8", None);
        add_link(&s.state, "c", "t", "https://github.com/o/new/pull/8", None);
        let server = FakeServer::serve(vec![ok(answer(&[
            raw_pull("https://github.com/o/r/pull/7", "OPEN"),
            raw_pull("https://github.com/o/new/pull/8", "OPEN"),
            raw_pull("https://github.com/o/new/pull/8", "OPEN"),
        ]))]);
        let github = GitHub::with_endpoint(&server.url).unwrap();

        refresh(
            &s.state,
            &providers(&github, &FakeTokens::default()),
            &Target::Day(day()),
            false,
        )
        .await
        .unwrap();
        let moved = stored(&s.state, "a");
        assert_eq!(moved.kind, LinkKind::GithubPull);
        assert_eq!(moved.url, "https://github.com/o/r/pull/7");
        assert_eq!(moved.label, "r#7");
        let kept = stored(&s.state, "b");
        assert_eq!(kept.url, "https://github.com/o/old/pull/8");
        assert!(kept.snapshot.is_some());
    }

    #[tokio::test]
    async fn refresh_keeps_the_last_known_statuses_when_github_cannot_be_reached() {
        let s = setup(true);
        let open = issue(IssueState::Open);
        add_link(
            &s.state,
            "a",
            "t",
            "https://github.com/o/r/issues/1",
            Some((NOW, &open)),
        );
        let github = GitHub::with_endpoint(&FakeServer::unreachable_url()).unwrap();

        let outcome = refresh(
            &s.state,
            &providers(&github, &FakeTokens::default()),
            &Target::Link("a".into()),
            true,
        )
        .await
        .unwrap();
        assert_eq!(
            outcome,
            RefreshOutcome {
                refreshed: 0,
                problems: github_problem(IntegrationProblem::Offline)
            }
        );
        let link = stored(&s.state, "a");
        assert_eq!(link.snapshot, Some(open));
        assert_eq!(link.fetched_at.map(|at| at.as_second()), Some(NOW));
        assert_eq!(
            serde_json::to_value(outcome).unwrap(),
            json!({ "refreshed": 0, "problems": { "github": { "kind": "offline" } } })
        );
    }

    #[tokio::test]
    async fn refresh_does_nothing_without_a_github_connection() {
        let s = setup(false);
        add_link(&s.state, "a", "t", "https://github.com/o/r/issues/1", None);
        let server = FakeServer::serve(Vec::new());
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();

        let outcome = refresh(
            &s.state,
            &providers(&github, &tokens),
            &Target::Day(day()),
            true,
        )
        .await
        .unwrap();
        assert_eq!(outcome, RefreshOutcome::default());
        assert_eq!(tokens.asked(), 0);
        assert!(stored(&s.state, "a").fetched_at.is_none());
    }

    #[tokio::test]
    async fn preview_asks_github_about_pull_requests_and_issues_only() {
        let s = setup(true);
        let server = FakeServer::serve(vec![
            // A pull request pasted as an issue.
            ok(answer(&[raw_pull("https://github.com/o/r/pull/7", "OPEN")])),
            ok(answer(&[Value::Null])),
        ]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();

        let found = preview(
            &s.state,
            &providers(&github, &tokens),
            "github.com/o/r/issues/7",
        )
        .await
        .unwrap();
        assert_eq!(found.link.url, "https://github.com/o/r/pull/7");
        assert_eq!(found.link.kind, LinkKind::GithubPull);
        assert!(matches!(found.snapshot, Some(LinkSnapshot::PullRequest(_))));

        let hidden = preview(
            &s.state,
            &providers(&github, &tokens),
            "https://github.com/acme/infra/pull/311",
        )
        .await
        .unwrap();
        assert_eq!(hidden.link.label, "infra#311");
        assert_eq!(hidden.fetch_error, Some(FetchError::NoAccess));
        assert_eq!(hidden.snapshot, None);

        let page = preview(
            &s.state,
            &providers(&github, &tokens),
            "https://docs.rs/jiff",
        )
        .await
        .unwrap();
        assert_eq!(page.link.label, "docs.rs");
        assert_eq!(server.authorizations().len(), 2);
        assert!(preview(
            &s.state,
            &providers(&github, &tokens),
            "javascript:alert(1)"
        )
        .await
        .is_err());
        // Nothing was stored.
        let links: i64 = s
            .state
            .db()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM todo_links", [], |r| r.get(0))
            .unwrap();
        assert_eq!(links, 0);
    }

    #[tokio::test]
    async fn preview_without_a_github_connection_tells_what_the_url_does() {
        let s = setup(false);
        let server = FakeServer::serve(Vec::new());
        let github = GitHub::with_endpoint(&server.url).unwrap();

        let preview = preview(
            &s.state,
            &providers(&github, &FakeTokens::default()),
            "https://github.com/o/r/pull/7",
        )
        .await
        .unwrap();
        assert_eq!(preview.link.label, "r#7");
        assert_eq!(
            (preview.snapshot, preview.fetch_error, preview.problem),
            (None, None, None)
        );
        assert!(server.authorizations().is_empty());
    }

    #[tokio::test]
    async fn preview_tells_why_github_could_not_be_asked() {
        let s = setup(true);
        let github = GitHub::with_endpoint(&FakeServer::unreachable_url()).unwrap();
        let preview = preview(
            &s.state,
            &providers(&github, &FakeTokens::default()),
            "https://github.com/o/r/pull/7",
        )
        .await
        .unwrap();
        assert_eq!(preview.problem, Some(IntegrationProblem::Offline));
        assert_eq!(
            serde_json::to_value(&preview).unwrap(),
            json!({
                "link": {
                    "kind": "github_pull",
                    "url": "https://github.com/o/r/pull/7",
                    "label": "r#7",
                    "github": { "owner": "o", "repo": "r", "number": 7 },
                },
                "problem": { "kind": "offline" },
            })
        );
    }

    #[tokio::test]
    async fn refresh_remembers_when_the_connection_needs_the_user() {
        let s = setup(true);
        add_link(&s.state, "a", "t", "https://github.com/o/r/issues/1", None);
        let server = FakeServer::serve(vec![
            (401, Vec::new(), r#"{"message":"Bad credentials"}"#.into()),
            ok(answer(&[raw_issue(
                "https://github.com/o/r/issues/1",
                "OPEN",
            )])),
        ]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let signed_out = IntegrationProblem::SignedOut {
            login: "octocat".into(),
        };

        let outcome = refresh(
            &s.state,
            &providers(&github, &FakeTokens::default()),
            &Target::Day(day()),
            false,
        )
        .await
        .unwrap();
        assert_eq!(outcome.problems, github_problem(signed_out.clone()));
        assert_eq!(
            s.state.attention(Provider::GitHub, "octocat"),
            Some(signed_out)
        );

        // Signed in again since.
        refresh(
            &s.state,
            &providers(&github, &FakeTokens::default()),
            &Target::Day(day()),
            false,
        )
        .await
        .unwrap();
        assert_eq!(s.state.attention(Provider::GitHub, "octocat"), None);
    }

    /// Switches to another workspace while giving its token, as if the user did while GitHub was being asked.
    struct SwitchingTokens<'a> {
        state: &'a AppState,
        to: &'a str,
    }

    impl TokenSource for SwitchingTokens<'_> {
        async fn token(&self, _login: &str) -> std::result::Result<String, IntegrationProblem> {
            workspaces::set_active(&self.state.app_db(), self.to, NOW).unwrap();
            Ok("t".into())
        }
    }

    #[tokio::test]
    async fn refresh_drops_what_github_said_once_another_workspace_is_active() {
        let s = setup(true);
        add_link(&s.state, "a", "t", "https://github.com/o/r/issues/1", None);
        let server = FakeServer::serve(vec![ok(answer(&[raw_issue(
            "https://github.com/o/r/issues/1",
            "OPEN",
        )]))]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = SwitchingTokens {
            state: &s.state,
            to: &s.other.id,
        };

        let outcome = refresh(
            &s.state,
            &providers(&github, &tokens),
            &Target::Day(day()),
            false,
        )
        .await
        .unwrap();
        assert_eq!(outcome, RefreshOutcome::default());
        assert_eq!(server.authorizations().len(), 1);
        assert!(stored(&s.state, "a").fetched_at.is_none());
    }
}
