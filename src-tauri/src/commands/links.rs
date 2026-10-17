use std::collections::HashMap;

use jiff::Timestamp;
use rusqlite::{named_params, Connection};
use tauri::State;
use uuid::Uuid;

use crate::{
    database::models::DbTodoLink,
    errors::{Result, TodaiError},
    links::{self, ParsedLink},
    models::{core::Day, integrations::TodoLink},
    state::AppState,
};

pub(crate) fn todo_not_found(id: &str) -> TodaiError {
    TodaiError::CommandError(format!("Todo {id} not found"))
}

pub(crate) fn link_not_found(id: &str) -> TodaiError {
    TodaiError::CommandError(format!("Link {id} not found"))
}

/// Recognizes what `url` points to, without adding it anywhere: see `add_todo_link`.
#[tauri::command]
pub async fn parse_link(url: String) -> Result<ParsedLink> {
    links::parse(&url)
}

/// Adds the links `urls` to the non-deleted todo `todo_id`, in this order after its other links, and returns them.
///
/// Each one must be an http(s) URL; they're normalized first (see `links::parse`). Fails without adding any if the todo
/// already has one of them, or if two of them are the same.
#[tauri::command]
pub async fn add_todo_links(
    state: State<'_, AppState>,
    todo_id: String,
    urls: Vec<String>,
) -> Result<Vec<TodoLink>> {
    let links = urls
        .iter()
        .map(|url| links::parse(url))
        .collect::<Result<Vec<_>>>()?;
    let mut db = state.db()?;
    insert_links(&mut db, &todo_id, links, Timestamp::now().as_second())
}

/// Removes the link `id` from its non-deleted todo.
#[tauri::command]
pub async fn remove_todo_link(state: State<'_, AppState>, id: String) -> Result<()> {
    let mut db = state.db()?;
    delete_link(&mut db, &id, Timestamp::now().as_second())
}

fn insert_links(
    db: &mut Connection,
    todo_id: &str,
    links: Vec<ParsedLink>,
    now: i64,
) -> Result<Vec<TodoLink>> {
    let tx = db.transaction()?;
    let inserted = insert_links_in(&tx, todo_id, links, now)?;
    tx.commit()?;
    Ok(inserted)
}

/// Adds `links` to the non-deleted todo `todo_id`, within the caller's transaction `tx`: see `add_todo_links`.
pub(crate) fn insert_links_in(
    tx: &Connection,
    todo_id: &str,
    links: Vec<ParsedLink>,
    now: i64,
) -> Result<Vec<TodoLink>> {
    if links.is_empty() {
        return Err(TodaiError::CommandError("No link to add".into()));
    }
    // Like its tags, its links are part of the todo.
    let touched = tx
        .prepare_cached("UPDATE todos SET updated_at = :now WHERE id = :id AND deleted_at IS NULL")?
        .execute(named_params! { ":id": todo_id, ":now": now })?;
    if touched == 0 {
        return Err(todo_not_found(todo_id));
    }
    let mut inserted = Vec::with_capacity(links.len());
    for link in links {
        // Also covers the links added just before.
        let exists: bool = tx
            .prepare_cached(
                "SELECT EXISTS (SELECT 1 FROM todo_links WHERE todo_id = ?1 AND url = ?2)",
            )?
            .query_row([todo_id, &link.url], |row| row.get(0))?;
        if exists {
            return Err(TodaiError::CommandError(format!(
                "The todo already links to {}",
                link.url
            )));
        }

        // The same Notion page, under another URL (e.g. without its slug)
        if let Some(notion) = &link.notion {
            let exists: bool = tx
                .prepare_cached(
                    "SELECT EXISTS (SELECT 1 FROM todo_links WHERE todo_id = ?1 AND notion_id = ?2)",
                )?
                .query_row([todo_id, &notion.id], |row| row.get(0))?;
            if exists {
                return Err(TodaiError::CommandError(format!(
                    "The todo already links to this Notion page ({})",
                    link.url
                )));
            }
        }

        let github = link.github.as_ref();
        let row = tx
            .prepare_cached(
                "INSERT INTO todo_links
                 (id, todo_id, kind, url, github_owner, github_repo, github_number, notion_id, created_at)
                 VALUES (:id, :todo_id, :kind, :url, :github_owner, :github_repo, :github_number, :notion_id, :now)
                 RETURNING *",
            )?
            .query_row(
                named_params! {
                    ":id": Uuid::now_v7().to_string(),
                    ":todo_id": todo_id,
                    ":kind": link.kind.to_db(),
                    ":url": link.url,
                    ":github_owner": github.map(|github| &github.owner),
                    ":github_repo": github.map(|github| &github.repo),
                    ":github_number": github.map(|github| github.number),
                    ":notion_id": link.notion.as_ref().map(|notion| &notion.id),
                    ":now": now,
                },
                DbTodoLink::from_row,
            )?;
        inserted.push(row.try_into()?);
    }
    Ok(inserted)
}

fn delete_link(db: &mut Connection, id: &str, now: i64) -> Result<()> {
    let tx = db.transaction()?;
    // Through its todo, which must not be deleted: links come back with it when restored.
    let touched = tx
        .prepare_cached(
            "UPDATE todos SET updated_at = :now
             WHERE id = (SELECT todo_id FROM todo_links WHERE id = :id) AND deleted_at IS NULL",
        )?
        .execute(named_params! { ":id": id, ":now": now })?;
    if touched == 0 {
        return Err(link_not_found(id));
    }
    tx.prepare_cached("DELETE FROM todo_links WHERE id = ?1")?
        .execute([id])?;
    tx.commit()?;
    Ok(())
}

/// The links of the non-deleted todos from `start` to `end` (inclusive), by todo ID, in the order they were added.
pub(crate) fn query_links_between(
    db: &Connection,
    start: &Day,
    end: &Day,
) -> Result<HashMap<String, Vec<TodoLink>>> {
    // Through `idx_todos_day`, then each todo's links through their `UNIQUE (todo_id, url)` index.
    let mut statement = db.prepare_cached(
        "SELECT todo_links.*
         FROM todos
         JOIN todo_links ON todo_links.todo_id = todos.id
         WHERE todos.day BETWEEN ?1 AND ?2 AND todos.deleted_at IS NULL
         ORDER BY todo_links.created_at, todo_links.id",
    )?;
    let rows = statement.query_map([&start.raw, &end.raw], DbTodoLink::from_row)?;
    let mut links: HashMap<String, Vec<TodoLink>> = HashMap::new();
    for row in rows {
        let link = TodoLink::try_from(row?)?;
        links.entry(link.todo_id.clone()).or_default().push(link);
    }
    Ok(links)
}

#[cfg(test)]
mod tests {
    use rusqlite::OptionalExtension;

    use super::*;
    use crate::{
        database::{open_test_database, WORKSPACE},
        links::LinkKind,
    };

    const NOW: i64 = 1_791_000_000;

    /// The link `id`, if any.
    fn query_link(db: &Connection, id: &str) -> Result<Option<TodoLink>> {
        db.prepare_cached("SELECT * FROM todo_links WHERE id = ?1")?
            .query_row([id], DbTodoLink::from_row)
            .optional()?
            .map(TodoLink::try_from)
            .transpose()
    }

    fn day(raw: &str) -> Day {
        raw.to_string().try_into().unwrap()
    }

    fn insert_todo(db: &Connection, id: &str, day_raw: &str) {
        db.execute(
            "INSERT INTO todos (id, day, title, position, created_at, updated_at) VALUES (?1, ?2, ?1, 'a0', 0, 0)",
            (id, day_raw),
        )
        .unwrap();
    }

    fn add(db: &mut Connection, todo_id: &str, url: &str, now: i64) -> Result<TodoLink> {
        let mut added = insert_links(db, todo_id, vec![links::parse(url).unwrap()], now)?;
        Ok(added.remove(0))
    }

    fn todo_updated_at(db: &Connection, id: &str) -> i64 {
        db.query_row("SELECT updated_at FROM todos WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
    }

    fn links_of(db: &Connection, todo_id: &str, day_raw: &str) -> Vec<String> {
        query_links_between(db, &day(day_raw), &day(day_raw))
            .unwrap()
            .remove(todo_id)
            .unwrap_or_default()
            .into_iter()
            .map(|link| link.label)
            .collect()
    }

    #[test]
    fn add_todo_link_stores_the_recognized_link_and_touches_its_todo() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");

        let link = add(
            &mut db,
            "t",
            "github.com/pierreyoda/todai/pull/58/files",
            NOW,
        )
        .unwrap();
        assert_eq!(link.todo_id, "t");
        assert_eq!(link.kind, LinkKind::GithubPull);
        assert_eq!(link.url, "https://github.com/pierreyoda/todai/pull/58");
        assert_eq!(link.label, "todai#58");
        assert_eq!(link.github.as_ref().unwrap().owner, "pierreyoda");
        assert_eq!(link.created_at.as_second(), NOW);
        assert_eq!(todo_updated_at(&db, "t"), NOW);

        let stored = query_link(&db, &link.id).unwrap().unwrap();
        assert_eq!(stored.url, link.url);
        assert_eq!(stored.github, link.github);
    }

    #[test]
    fn add_todo_link_rejects_a_link_the_todo_already_has() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        insert_todo(&db, "u", "2026-10-10");
        add(
            &mut db,
            "t",
            "https://github.com/pierreyoda/todai/pull/58",
            NOW,
        )
        .unwrap();

        // The same pull request, on another tab.
        let result = add(
            &mut db,
            "t",
            "https://github.com/pierreyoda/todai/pull/58/commits",
            NOW,
        );
        assert!(matches!(result, Err(TodaiError::CommandError(_))));
        // Another todo can have it too.
        add(
            &mut db,
            "u",
            "https://github.com/pierreyoda/todai/pull/58",
            NOW,
        )
        .unwrap();
        assert_eq!(links_of(&db, "t", "2026-10-10"), ["todai#58"]);
    }

    #[test]
    fn add_todo_links_adds_several_in_order_or_none() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        let parsed = |url: &str| links::parse(url).unwrap();

        let added = insert_links(
            &mut db,
            "t",
            vec![
                parsed("https://github.com/o/r/issues/12"),
                parsed("https://github.com/o/r/pull/58"),
            ],
            NOW,
        )
        .unwrap();
        assert_eq!(added.len(), 2);
        assert_eq!(links_of(&db, "t", "2026-10-10"), ["r#12", "r#58"]);

        // One already there: none is added.
        let result = insert_links(
            &mut db,
            "t",
            vec![
                parsed("https://example.org"),
                parsed("https://github.com/o/r/pull/58"),
            ],
            NOW,
        );
        assert!(matches!(result, Err(TodaiError::CommandError(_))));
        // The same twice.
        let result = insert_links(
            &mut db,
            "t",
            vec![
                parsed("https://example.org"),
                parsed("https://example.org/"),
            ],
            NOW,
        );
        assert!(matches!(result, Err(TodaiError::CommandError(_))));
        assert!(insert_links(&mut db, "t", Vec::new(), NOW).is_err());
        assert_eq!(links_of(&db, "t", "2026-10-10"), ["r#12", "r#58"]);
    }

    #[test]
    fn add_todo_link_rejects_deleted_and_unknown_todos() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        db.execute("UPDATE todos SET deleted_at = 1 WHERE id = 't'", [])
            .unwrap();
        assert!(add(&mut db, "t", "https://example.org", NOW).is_err());
        assert!(add(&mut db, "unknown", "https://example.org", NOW).is_err());
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM todo_links", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn remove_todo_link_deletes_it_and_touches_its_todo() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        let link = add(&mut db, "t", "https://example.org", NOW).unwrap();
        add(&mut db, "t", "https://github.com/o/r/issues/1", NOW).unwrap();

        delete_link(&mut db, &link.id, NOW + 60).unwrap();
        assert!(query_link(&db, &link.id).unwrap().is_none());
        assert_eq!(links_of(&db, "t", "2026-10-10"), ["r#1"]);
        assert_eq!(todo_updated_at(&db, "t"), NOW + 60);

        assert!(delete_link(&mut db, &link.id, NOW).is_err());
    }

    #[test]
    fn remove_todo_link_leaves_the_links_of_deleted_todos() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        let link = add(&mut db, "t", "https://example.org", NOW).unwrap();
        db.execute("UPDATE todos SET deleted_at = 1 WHERE id = 't'", [])
            .unwrap();
        assert!(delete_link(&mut db, &link.id, NOW).is_err());
        assert!(query_link(&db, &link.id).unwrap().is_some());
    }

    #[test]
    fn query_links_between_groups_them_by_todo_in_the_order_they_were_added() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "a", "2026-10-09");
        insert_todo(&db, "b", "2026-10-10");
        insert_todo(&db, "c", "2026-10-11");
        add(&mut db, "b", "https://github.com/o/r/pull/2", NOW).unwrap();
        add(&mut db, "a", "https://example.org", NOW).unwrap();
        add(&mut db, "b", "https://github.com/o/r/issues/1", NOW).unwrap();
        add(&mut db, "c", "https://example.com", NOW).unwrap();

        let links = query_links_between(&db, &day("2026-10-09"), &day("2026-10-10")).unwrap();
        assert_eq!(links.len(), 2);
        let labels =
            |id: &str| -> Vec<&str> { links[id].iter().map(|l| l.label.as_str()).collect() };
        assert_eq!(labels("a"), ["example.org"]);
        assert_eq!(labels("b"), ["r#2", "r#1"]);
    }

    #[test]
    fn deleted_todos_keep_their_links_for_when_they_are_restored() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        add(&mut db, "t", "https://example.org", NOW).unwrap();

        db.execute("UPDATE todos SET deleted_at = 1 WHERE id = 't'", [])
            .unwrap();
        assert!(links_of(&db, "t", "2026-10-10").is_empty());

        db.execute("UPDATE todos SET deleted_at = NULL WHERE id = 't'", [])
            .unwrap();
        assert_eq!(links_of(&db, "t", "2026-10-10"), ["example.org"]);
    }

    #[test]
    fn links_follow_their_todo_to_another_day_and_go_with_it() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        add(&mut db, "t", "https://example.org", NOW).unwrap();

        db.execute("UPDATE todos SET day = '2026-10-11' WHERE id = 't'", [])
            .unwrap();
        assert_eq!(links_of(&db, "t", "2026-10-11"), ["example.org"]);

        db.execute("DELETE FROM todos WHERE id = 't'", []).unwrap();
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM todo_links", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn the_database_rejects_inconsistent_github_columns() {
        let db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        let insert = |kind: &str, owner: Option<&str>, number: Option<u32>| {
            db.execute(
                "INSERT INTO todo_links (id, todo_id, kind, url, github_owner, github_repo, github_number, created_at)
                 VALUES (?1, 't', ?2, ?1, ?3, ?3, ?4, 0)",
                (Uuid::now_v7().to_string(), kind, owner, number),
            )
        };
        assert!(insert("github_pull", None, None).is_err());
        assert!(insert("github_issue", Some("o"), None).is_err());
        assert!(insert("github_pull", Some("o"), Some(0)).is_err());
        assert!(insert("url", Some("o"), Some(1)).is_err());
        assert!(insert("notion_page", None, None).is_err());
        assert!(insert("github_pull", Some("o"), Some(1)).is_ok());
        assert!(insert("url", None, None).is_ok());
    }

    #[test]
    fn todo_link_serializes_in_camel_case_without_absent_github() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "t", "2026-10-10");
        let pull = add(&mut db, "t", "https://github.com/o/r/pull/7", NOW).unwrap();
        let mut value = serde_json::to_value(&pull).unwrap();
        assert_eq!(value["id"], serde_json::json!(pull.id));
        value.as_object_mut().unwrap().remove("id");
        assert_eq!(
            value,
            serde_json::json!({
                "todoId": "t",
                "kind": "github_pull",
                "url": "https://github.com/o/r/pull/7",
                "label": "r#7",
                "github": { "owner": "o", "repo": "r", "number": 7 },
                "createdAt": "2026-10-03T04:00:00Z",
            })
        );
        let page = add(&mut db, "t", "https://www.example.org/a", NOW).unwrap();
        let value = serde_json::to_value(&page).unwrap();
        assert_eq!(value["label"], "example.org");
        assert!(value.get("github").is_none());
    }

    #[test]
    fn notion_pages_are_added_with_their_id_and_once_whatever_their_url() {
        let mut db = open_test_database(&WORKSPACE);
        insert_todo(&db, "plan", "2026-10-10");
        let id = "8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5";
        let links = insert_links(
            &mut db,
            "plan",
            vec![links::parse(&format!("https://www.notion.so/acme/Roadmap-{id}?pvs=4")).unwrap()],
            NOW,
        )
        .unwrap();
        assert_eq!(links[0].kind, LinkKind::Notion);
        assert_eq!(links[0].label, "Roadmap");
        assert_eq!(links[0].notion.as_ref().unwrap().id, id);
        assert_eq!(links[0].url, format!("https://www.notion.so/Roadmap-{id}"));

        // The same page, without its slug
        let again = insert_links(
            &mut db,
            "plan",
            vec![links::parse(&format!("https://app.notion.com/p/{id}")).unwrap()],
            NOW,
        );
        assert!(again.is_err());
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM todo_links", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
