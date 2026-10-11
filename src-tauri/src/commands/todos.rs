use std::collections::HashSet;

use fractional_index::FractionalIndex;
use jiff::Timestamp;
use rusqlite::{named_params, Connection, OptionalExtension};
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::{
    commands::links::{insert_links_in, query_links_between},
    database::models::{DbDay, DbTodo},
    errors::{Result, TodaiError},
    links::{self, ParsedLink},
    models::core::{Day, Estimate, Todo, TodoMonth},
    state::AppState,
};

/// Lists the non-deleted todos of `day`, in display order, with the IDs of their non-deleted tags and their links.
#[tauri::command]
pub async fn list_todos(state: State<'_, AppState>, day: Day) -> Result<Vec<Todo>> {
    let db = state.db()?;
    query_todos(&db, &day, &day)
}

/// Lists the non-deleted todos from `start` to `end` (inclusive), by day then in display order, as `list_todos`.
#[tauri::command]
pub async fn list_todos_between(
    state: State<'_, AppState>,
    start: Day,
    end: Day,
) -> Result<Vec<Todo>> {
    if start > end {
        return Err(TodaiError::CommandError(format!(
            "Start day {} is after end day {}",
            start.raw, end.raw
        )));
    }
    let db = state.db()?;
    query_todos(&db, &start, &end)
}

/// Lists the months having non-deleted todos, most recent first, with their statistics.
#[tauri::command]
pub async fn list_todo_months(state: State<'_, AppState>) -> Result<Vec<TodoMonth>> {
    let db = state.db()?;
    // `day` is YYYY-MM-DD: its first 7 characters are the month. Covered by `idx_todos_day`.
    let mut statement = db.prepare_cached(
        "SELECT substr(day, 1, 7) AS month,
                COUNT(*) AS count,
                SUM(completed) AS completed_count,
                COALESCE(SUM(estimate) FILTER (WHERE estimate_unit = 'minutes'), 0) AS estimated_minutes,
                COALESCE(SUM(estimate) FILTER (WHERE estimate_unit = 'points'), 0) AS estimated_points
         FROM todos
         WHERE deleted_at IS NULL
         GROUP BY month
         ORDER BY month DESC",
    )?;
    let months = statement
        .query_map([], |row| {
            Ok(TodoMonth {
                month: row.get("month")?,
                count: row.get("count")?,
                completed_count: row.get("completed_count")?,
                estimated_minutes: row.get("estimated_minutes")?,
                estimated_points: row.get("estimated_points")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(months)
}

/// The non-deleted todos from `start` to `end` (inclusive), by day then in display order, with their tags and links.
fn query_todos(db: &Connection, start: &Day, end: &Day) -> Result<Vec<Todo>> {
    // One row per (todo, tag) pair, or a single row with a NULL tag ID for an untagged todo.
    // Tags are only joined to skip deleted ones. Sorting by `todos.id` after `position` keeps each todo's rows contiguous.
    let mut statement = db.prepare_cached(
        "SELECT todos.*, tags.id AS tag_id
         FROM todos
         LEFT JOIN todo_tags ON todo_tags.todo_id = todos.id
         LEFT JOIN tags ON tags.id = todo_tags.tag_id AND tags.deleted_at IS NULL
         WHERE todos.day BETWEEN ?1 AND ?2 AND todos.deleted_at IS NULL
         ORDER BY todos.day, todos.position, todos.id",
    )?;
    let rows = statement.query_map([&start.raw, &end.raw], |row| {
        Ok((
            DbTodo::from_row(row)?,
            row.get::<_, Option<String>>("tag_id")?,
        ))
    })?;

    let mut todos: Vec<Todo> = Vec::new();
    for row in rows {
        let (todo, tag_id) = row?;
        let todo = match todos.last_mut() {
            Some(last) if last.id == todo.id => last,
            _ => {
                todos.push(todo.try_into()?);
                todos.last_mut().expect("just pushed")
            }
        };
        if let Some(tag_id) = tag_id {
            todo.tag_ids.push(tag_id);
        }
    }

    // In a second query rather than joined: a todo with several tags and links would otherwise get a row per pair.
    let mut links = query_links_between(db, start, end)?;
    for todo in &mut todos {
        todo.links = links.remove(&todo.id).unwrap_or_default();
    }
    Ok(todos)
}

/// A position after every todo of `day`, to add one at its end.
fn position_after_last(db: &Connection, day: &Day) -> Result<FractionalIndex> {
    // Includes deleted todos, so that restoring one never collides with a newer position.
    let last_position: Option<String> = db
        .prepare_cached("SELECT MAX(position) FROM todos WHERE day = ?1")?
        .query_row([&day.raw], |row| row.get(0))?;
    Ok(match last_position {
        Some(last) => FractionalIndex::new_after(
            &FractionalIndex::from_string(&last).map_err(|_| TodaiError::InvalidPosition(last))?,
        ),
        None => FractionalIndex::default(),
    })
}

/// Creates a todo at the end of `day`.
///
/// `title` and `description` are trimmed; an empty `description` is stored as `NULL`.
/// `estimate` is checked as in `set_todo_estimate`.
#[tauri::command]
pub async fn create_todo(
    state: State<'_, AppState>,
    day: Day,
    title: String,
    description: Option<String>,
    estimate: Option<Estimate>,
) -> Result<Todo> {
    let description = description
        .as_deref()
        .map(str::trim)
        .filter(|description| !description.is_empty());
    let estimate = estimate.map(Estimate::validate).transpose()?;
    let db = state.db()?;
    insert_new_todo(
        &db,
        &day,
        &title,
        description,
        estimate,
        Timestamp::now().as_second(),
    )
}

/// Creates a todo at the end of `day` with the link `url`, at once: titled `title` (e.g. its pull request's, as GitHub
/// tells through `preview_link`), or else as the link is shown (e.g. `todai#58`).
#[tauri::command]
pub async fn create_todo_from_link(
    state: State<'_, AppState>,
    day: Day,
    url: String,
    title: Option<String>,
) -> Result<Todo> {
    let link = links::parse(&url)?;
    let title = title
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| link.label.clone());
    let mut db = state.db()?;
    create_with_link(&mut db, &day, &title, link, Timestamp::now().as_second())
}

fn create_with_link(
    db: &mut Connection,
    day: &Day,
    title: &str,
    link: ParsedLink,
    now: i64,
) -> Result<Todo> {
    let tx = db.transaction()?;
    let mut todo = insert_new_todo(&tx, day, title, None, None, now)?;
    todo.links = insert_links_in(&tx, &todo.id, vec![link], now)?;
    tx.commit()?;
    Ok(todo)
}

/// Inserts a todo at the end of `day`, created at `now`. Its `title` is trimmed, and must not be blank.
fn insert_new_todo(
    db: &Connection,
    day: &Day,
    title: &str,
    description: Option<&str>,
    estimate: Option<Estimate>,
    now: i64,
) -> Result<Todo> {
    let title = title.trim();
    if title.is_empty() {
        return Err(TodaiError::CommandError(
            "Todo title cannot be empty".into(),
        ));
    }
    let (estimate, estimate_unit) = estimate.map(Estimate::to_db).unzip();
    let position = position_after_last(db, day)?;
    let todo = db
        .prepare_cached(
            "INSERT INTO todos (id, day, title, description, position, estimate, estimate_unit, created_at, updated_at)
             VALUES (:id, :day, :title, :description, :position, :estimate, :estimate_unit, :now, :now)
             RETURNING *",
        )?
        .query_row(
            named_params! {
                ":id": Uuid::now_v7().to_string(),
                ":day": day.raw,
                ":title": title,
                ":description": description,
                ":position": position.to_string(),
                ":estimate": estimate,
                ":estimate_unit": estimate_unit,
                ":now": now,
            },
            DbTodo::from_row,
        )?;
    todo.try_into()
}

/// The parameters of `update_todo`: fields left to `None` are unchanged.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTodoParams {
    pub id: String,
    /// Trimmed and not empty, as in `create_todo`.
    pub title: Option<String>,
    /// Moving a todo to another day puts it at the end of that day.
    pub day: Option<Day>,
    /// Sets or clears `completed_at`, as in `toggle_todo`, only if it changes.
    pub completed: Option<bool>,
}

/// Updates a non-deleted todo's `title`, `day` and/or `completed`.
#[tauri::command]
pub async fn update_todo(state: State<'_, AppState>, params: UpdateTodoParams) -> Result<()> {
    let db = state.db()?;
    update_todo_row(&db, params, Timestamp::now().as_second())
}

fn update_todo_row(db: &Connection, params: UpdateTodoParams, now: i64) -> Result<()> {
    let UpdateTodoParams {
        id,
        title,
        day,
        completed,
    } = params;
    let title = title.as_deref().map(str::trim);
    if title.is_some_and(str::is_empty) {
        return Err(TodaiError::CommandError(
            "Todo title cannot be empty".into(),
        ));
    }
    let not_found = || TodaiError::CommandError(format!("Todo {id} not found"));

    // Only a todo moved to another day gets a new position.
    let position = match &day {
        None => None,
        Some(day) => {
            let current_day: DbDay = db
                .prepare_cached("SELECT day FROM todos WHERE id = ?1 AND deleted_at IS NULL")?
                .query_row([&id], |row| row.get(0))
                .optional()?
                .ok_or_else(not_found)?;
            (current_day != day.raw)
                .then(|| position_after_last(db, day))
                .transpose()?
        }
    };

    // Expressions in `SET` read the row's values from before the update.
    let updated = db
        .prepare_cached(
            "UPDATE todos
                 SET title = COALESCE(:title, title),
                     day = COALESCE(:day, day),
                     position = COALESCE(:position, position),
                     completed = COALESCE(:completed, completed),
                     completed_at = CASE
                         WHEN :completed IS NULL OR :completed = completed THEN completed_at
                         WHEN :completed THEN :now
                     END,
                     updated_at = :now
                 WHERE id = :id AND deleted_at IS NULL",
        )?
        .execute(named_params! {
            ":id": id,
            ":title": title,
            ":day": day.map(DbDay::from),
            ":position": position.map(|position| position.to_string()),
            ":completed": completed,
            ":now": now,
        })?;
    if updated == 0 {
        return Err(not_found());
    }
    Ok(())
}

/// Moves the non-deleted todos `ids` to the end of `day`, in this order, as `update_todo` would one by one: those already
/// on `day` stay in place.
///
/// Fails without moving any if one of them is deleted or unknown.
#[tauri::command]
pub async fn move_todos(state: State<'_, AppState>, ids: Vec<String>, day: Day) -> Result<()> {
    let mut db = state.db()?;
    move_todo_rows(&mut db, ids, &day, Timestamp::now().as_second())
}

fn move_todo_rows(db: &mut Connection, ids: Vec<String>, day: &Day, now: i64) -> Result<()> {
    let tx = db.transaction()?;
    for id in ids {
        let params = UpdateTodoParams {
            id,
            title: None,
            day: Some(day.clone()),
            completed: None,
        };
        update_todo_row(&tx, params, now)?;
    }
    tx.commit()?;
    Ok(())
}

/// Sets a non-deleted todo's estimate, replacing any previous one (whatever its unit), or removes it if `None`.
///
/// The value must be positive and at most `Estimate::MAX_MINUTES` or `Estimate::MAX_POINTS`, depending on its unit.
#[tauri::command]
pub async fn set_todo_estimate(
    state: State<'_, AppState>,
    id: String,
    estimate: Option<Estimate>,
) -> Result<()> {
    let (estimate, estimate_unit) = estimate
        .map(Estimate::validate)
        .transpose()?
        .map(Estimate::to_db)
        .unzip();

    let db = state.db()?;
    let now = Timestamp::now().as_second();
    let updated = db
        .prepare_cached(
            "UPDATE todos
                 SET estimate = :estimate,
                     estimate_unit = :estimate_unit,
                     updated_at = :now
                 WHERE id = :id AND deleted_at IS NULL",
        )?
        .execute(named_params! {
            ":id": id,
            ":estimate": estimate,
            ":estimate_unit": estimate_unit,
            ":now": now,
        })?;
    if updated == 0 {
        return Err(TodaiError::CommandError(format!("Todo {id} not found")));
    }
    Ok(())
}

/// Update a todo's completeness status.
#[tauri::command]
pub async fn toggle_todo(state: State<'_, AppState>, id: String, completed: bool) -> Result<()> {
    let db = state.db()?;
    let now = Timestamp::now().as_second();
    let updated = db
        .prepare_cached(
            "UPDATE todos
                 SET completed = :completed,
                     completed_at = :completed_at,
                     updated_at = :now
                 WHERE id = :id",
        )?
        .execute(named_params! {
            ":id": id,
            ":completed": completed as i32,
            ":completed_at": if completed { Some(now) } else { None },
            ":now": now,
        })?;
    if updated == 0 {
        return Err(TodaiError::CommandError(format!("Todo {id} not found")));
    }
    Ok(())
}

/// Deletes a todo, keeping its row and links to tags so that it can be restored.
#[tauri::command]
pub async fn delete_todo(state: State<'_, AppState>, id: String) -> Result<()> {
    let db = state.db()?;
    let now = Timestamp::now().as_second();
    let deleted = db
        .prepare_cached(
            "UPDATE todos
                 SET deleted_at = :now,
                     updated_at = :now
                 WHERE id = :id AND deleted_at IS NULL",
        )?
        .execute(named_params! { ":id": id, ":now": now })?;
    if deleted == 0 {
        return Err(TodaiError::CommandError(format!("Todo {id} not found")));
    }
    Ok(())
}

/// Restores a deleted todo to its day, at its former position, along with `tag_ids`: some of its deleted tags, which
/// then come back on their other todos too.
///
/// Fails without restoring anything if the todo is not deleted, or if any of `tag_ids` is not one of its deleted tags.
#[tauri::command]
pub async fn restore_todo(
    state: State<'_, AppState>,
    id: String,
    tag_ids: Vec<String>,
) -> Result<()> {
    let mut db = state.db()?;
    restore_todo_row(&mut db, &id, &tag_ids, Timestamp::now().as_second())
}

fn restore_todo_row(db: &mut Connection, id: &str, tag_ids: &[String], now: i64) -> Result<()> {
    let tag_ids: HashSet<&str> = tag_ids.iter().map(String::as_str).collect();

    let tx = db.transaction()?;
    // Its position was kept: `position_after_last` counts deleted todos, so no other todo of its day took it since.
    let restored = tx
        .prepare_cached(
            "UPDATE todos
                 SET deleted_at = NULL,
                     updated_at = :now
                 WHERE id = :id AND deleted_at IS NOT NULL",
        )?
        .execute(named_params! { ":id": id, ":now": now })?;
    if restored == 0 {
        return Err(TodaiError::CommandError(format!(
            "Deleted todo {id} not found"
        )));
    }
    {
        let mut restore_tag = tx.prepare_cached(
            "UPDATE tags
                 SET deleted_at = NULL,
                     updated_at = :now
                 WHERE id = :tag_id
                   AND deleted_at IS NOT NULL
                   AND id IN (SELECT tag_id FROM todo_tags WHERE todo_id = :todo_id)",
        )?;
        for tag_id in tag_ids {
            let restored = restore_tag.execute(named_params! {
                ":tag_id": tag_id,
                ":todo_id": id,
                ":now": now,
            })?;
            if restored == 0 {
                return Err(TodaiError::CommandError(format!(
                    "Tag {tag_id} is not a deleted tag of todo {id}"
                )));
            }
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::{open_test_database, WORKSPACE},
        models::core::Day,
    };

    const NOW: i64 = 1_791_000_000;

    fn day(raw: &str) -> Day {
        raw.to_string().try_into().unwrap()
    }

    /// Adds a todo at the end of `day`, returning its ID.
    fn insert_todo(db: &Connection, day_raw: &str, title: &str) -> String {
        let id = Uuid::now_v7().to_string();
        let position = position_after_last(db, &day(day_raw)).unwrap().to_string();
        db.execute(
            "INSERT INTO todos (id, day, title, position, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 0, 0)",
            (&id, day_raw, title, position),
        )
        .unwrap();
        id
    }

    fn todo_row(db: &Connection, id: &str) -> DbTodo {
        db.query_row("SELECT * FROM todos WHERE id = ?1", [id], DbTodo::from_row)
            .unwrap()
    }

    fn params(id: &str) -> UpdateTodoParams {
        UpdateTodoParams {
            id: id.to_string(),
            title: None,
            day: None,
            completed: None,
        }
    }

    #[test]
    fn update_todo_params_deserialize_from_camel_case_with_optional_fields() {
        let params: UpdateTodoParams = serde_json::from_value(serde_json::json!({
            "id": "a",
            "day": "2026-10-04",
            "completed": true,
        }))
        .unwrap();
        assert_eq!(params.id, "a");
        assert_eq!(params.title, None);
        assert_eq!(params.day, Some(day("2026-10-04")));
        assert_eq!(params.completed, Some(true));
        assert!(serde_json::from_value::<UpdateTodoParams>(
            serde_json::json!({ "id": "a", "day": "20261004" })
        )
        .is_err());
    }

    #[test]
    fn update_todo_trims_the_title_and_leaves_other_fields() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Before");
        let before = todo_row(&db, &id);
        update_todo_row(
            &db,
            UpdateTodoParams {
                title: Some("  After ".into()),
                ..params(&id)
            },
            NOW,
        )
        .unwrap();
        let after = todo_row(&db, &id);
        assert_eq!(after.title, "After");
        assert_eq!(after.day, before.day);
        assert_eq!(after.position, before.position);
        assert_eq!(after.completed, 0);
        assert_eq!(after.updated_at, NOW);
    }

    #[test]
    fn update_todo_rejects_an_empty_title() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Title");
        let result = update_todo_row(
            &db,
            UpdateTodoParams {
                title: Some("   ".into()),
                ..params(&id)
            },
            NOW,
        );
        assert!(result.is_err());
        assert_eq!(todo_row(&db, &id).title, "Title");
    }

    #[test]
    fn update_todo_moves_a_todo_to_the_end_of_another_day() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Moved");
        let other = insert_todo(&db, "2026-10-05", "Already there");
        update_todo_row(
            &db,
            UpdateTodoParams {
                day: Some(day("2026-10-05")),
                ..params(&id)
            },
            NOW,
        )
        .unwrap();
        let moved = todo_row(&db, &id);
        assert_eq!(moved.day, "2026-10-05");
        assert!(moved.position > todo_row(&db, &other).position);
    }

    #[test]
    fn update_todo_keeps_the_position_on_the_same_day() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "First");
        insert_todo(&db, "2026-10-04", "Second");
        let before = todo_row(&db, &id);
        update_todo_row(
            &db,
            UpdateTodoParams {
                day: Some(day("2026-10-04")),
                ..params(&id)
            },
            NOW,
        )
        .unwrap();
        assert_eq!(todo_row(&db, &id).position, before.position);
    }

    #[test]
    fn update_todo_sets_completed_at_only_when_completed_changes() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Title");
        let complete = |completed, now| {
            update_todo_row(
                &db,
                UpdateTodoParams {
                    completed: Some(completed),
                    ..params(&id)
                },
                now,
            )
            .unwrap();
            let todo = todo_row(&db, &id);
            (todo.completed, todo.completed_at)
        };
        assert_eq!(complete(true, NOW), (1, Some(NOW)));
        // Already completed: keeps the original completion time
        assert_eq!(complete(true, NOW + 60), (1, Some(NOW)));
        assert_eq!(complete(false, NOW + 120), (0, None));
        assert_eq!(complete(false, NOW + 180), (0, None));
    }

    #[test]
    fn update_todo_without_completed_keeps_completed_at() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Title");
        update_todo_row(
            &db,
            UpdateTodoParams {
                completed: Some(true),
                ..params(&id)
            },
            NOW,
        )
        .unwrap();
        update_todo_row(
            &db,
            UpdateTodoParams {
                title: Some("Renamed".into()),
                ..params(&id)
            },
            NOW + 60,
        )
        .unwrap();
        let todo = todo_row(&db, &id);
        assert_eq!((todo.completed, todo.completed_at), (1, Some(NOW)));
    }

    #[test]
    fn update_todo_ignores_deleted_and_unknown_todos() {
        let db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Deleted");
        db.execute("UPDATE todos SET deleted_at = 1 WHERE id = ?1", [&id])
            .unwrap();
        let rename = UpdateTodoParams {
            title: Some("Renamed".into()),
            ..params(&id)
        };
        assert!(update_todo_row(&db, rename, NOW).is_err());
        let moved = UpdateTodoParams {
            day: Some(day("2026-10-05")),
            ..params(&id)
        };
        assert!(update_todo_row(&db, moved, NOW).is_err());
        assert_eq!(todo_row(&db, &id).title, "Deleted");
        assert!(update_todo_row(&db, params("unknown"), NOW).is_err());
    }

    #[test]
    fn move_todos_moves_them_in_order_to_the_end_of_the_day() {
        let mut db = open_test_database(&WORKSPACE);
        let older = insert_todo(&db, "2026-10-04", "Older");
        let newer = insert_todo(&db, "2026-10-06", "Newer");
        let today = insert_todo(&db, "2026-10-07", "Today");
        move_todo_rows(
            &mut db,
            vec![older.clone(), newer.clone(), today.clone()],
            &day("2026-10-07"),
            NOW,
        )
        .unwrap();
        let listed: Vec<String> = query_todos(&db, &day("2026-10-07"), &day("2026-10-07"))
            .unwrap()
            .into_iter()
            .map(|todo| todo.id)
            .collect();
        // The one already there keeps its place, before the moved ones
        assert_eq!(listed, [today, older, newer]);
    }

    #[test]
    fn move_todos_moves_none_if_one_is_deleted_or_unknown() {
        let mut db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Kept");
        let deleted = insert_todo(&db, "2026-10-04", "Deleted");
        db.execute("UPDATE todos SET deleted_at = 1 WHERE id = ?1", [&deleted])
            .unwrap();
        for other in [deleted, "unknown".to_string()] {
            let result = move_todo_rows(&mut db, vec![id.clone(), other], &day("2026-10-07"), NOW);
            assert!(result.is_err());
            assert_eq!(todo_row(&db, &id).day, "2026-10-04");
        }
    }

    /// Adds a tag, returning its ID.
    fn insert_tag(db: &Connection, name: &str) -> String {
        let id = Uuid::now_v7().to_string();
        db.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES (?1, ?2, 0, 0, 0)",
            (&id, name),
        )
        .unwrap();
        id
    }

    fn tag_todo(db: &Connection, todo_id: &str, tag_id: &str) {
        db.execute(
            "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
            [todo_id, tag_id],
        )
        .unwrap();
    }

    /// Deletes the row `id` of `table` (`todos` or `tags`) at `at`, as `delete_todo` and `delete_tag` do.
    fn delete_at(db: &Connection, table: &str, id: &str, at: i64) {
        db.execute(
            &format!("UPDATE {table} SET deleted_at = ?2 WHERE id = ?1"),
            (id, at),
        )
        .unwrap();
    }

    fn tag_deleted_at(db: &Connection, id: &str) -> Option<i64> {
        db.query_row("SELECT deleted_at FROM tags WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
    }

    fn ids(todos: &[Todo]) -> Vec<&str> {
        todos.iter().map(|todo| todo.id.as_str()).collect()
    }

    fn day_todos(db: &Connection, day_raw: &str) -> Vec<Todo> {
        query_todos(db, &day(day_raw), &day(day_raw)).unwrap()
    }

    #[test]
    fn restore_todo_puts_it_back_at_its_position() {
        let mut db = open_test_database(&WORKSPACE);
        let first = insert_todo(&db, "2026-10-04", "First");
        let last = insert_todo(&db, "2026-10-04", "Last");
        delete_at(&db, "todos", &last, NOW);
        // Added once it was deleted: still after it.
        let added = insert_todo(&db, "2026-10-04", "Added");

        restore_todo_row(&mut db, &last, &[], NOW + 60).unwrap();
        assert_eq!(
            ids(&day_todos(&db, "2026-10-04")),
            [first.as_str(), last.as_str(), added.as_str()]
        );
        let restored = todo_row(&db, &last);
        assert_eq!(restored.deleted_at, None);
        assert_eq!(restored.updated_at, NOW + 60);
    }

    #[test]
    fn restore_todo_restores_the_given_deleted_tags() {
        let mut db = open_test_database(&WORKSPACE);
        let urgent = insert_tag(&db, "urgent");
        let errands = insert_tag(&db, "errands");
        let id = insert_todo(&db, "2026-10-04", "Restored");
        let other = insert_todo(&db, "2026-10-05", "Other");
        tag_todo(&db, &id, &urgent);
        tag_todo(&db, &id, &errands);
        tag_todo(&db, &other, &urgent);
        delete_at(&db, "tags", &urgent, NOW);
        delete_at(&db, "tags", &errands, NOW);
        delete_at(&db, "todos", &id, NOW);

        restore_todo_row(&mut db, &id, std::slice::from_ref(&urgent), NOW + 60).unwrap();
        assert_eq!(tag_deleted_at(&db, &urgent), None);
        assert_eq!(tag_deleted_at(&db, &errands), Some(NOW));
        // Back on its other todo too, while the tag left deleted stays linked, hidden.
        assert_eq!(day_todos(&db, "2026-10-04")[0].tag_ids, [urgent.as_str()]);
        assert_eq!(day_todos(&db, "2026-10-05")[0].tag_ids, [urgent.as_str()]);
    }

    #[test]
    fn restore_todo_fails_without_changes_for_other_tags() {
        let mut db = open_test_database(&WORKSPACE);
        let deleted_tag = insert_tag(&db, "deleted");
        let kept_tag = insert_tag(&db, "kept");
        let elsewhere = insert_tag(&db, "elsewhere");
        let id = insert_todo(&db, "2026-10-04", "Deleted");
        let other = insert_todo(&db, "2026-10-04", "Other");
        tag_todo(&db, &id, &deleted_tag);
        tag_todo(&db, &id, &kept_tag);
        tag_todo(&db, &other, &elsewhere);
        delete_at(&db, "tags", &deleted_tag, NOW);
        delete_at(&db, "tags", &elsewhere, NOW);
        delete_at(&db, "todos", &id, NOW);

        // Each along with its deleted tag: one of its non-deleted tags, a deleted tag of another todo, an unknown one.
        for invalid in [kept_tag, elsewhere.clone(), "unknown".into()] {
            let tag_ids = [deleted_tag.clone(), invalid.clone()];
            assert!(
                restore_todo_row(&mut db, &id, &tag_ids, NOW + 60).is_err(),
                "{invalid}"
            );
            assert_eq!(todo_row(&db, &id).deleted_at, Some(NOW));
            assert_eq!(tag_deleted_at(&db, &deleted_tag), Some(NOW));
        }
        assert_eq!(tag_deleted_at(&db, &elsewhere), Some(NOW));
    }

    #[test]
    fn restore_todo_fails_for_non_deleted_and_unknown_todos() {
        let mut db = open_test_database(&WORKSPACE);
        let id = insert_todo(&db, "2026-10-04", "Not deleted");
        assert!(restore_todo_row(&mut db, &id, &[], NOW).is_err());
        assert_eq!(todo_row(&db, &id).updated_at, 0);
        assert!(restore_todo_row(&mut db, "unknown", &[], NOW).is_err());
    }

    #[test]
    fn query_todos_includes_each_todo_tags_and_links_once() {
        let db = open_test_database(&WORKSPACE);
        let linked = insert_todo(&db, "2026-10-10", "Linked");
        let unlinked = insert_todo(&db, "2026-10-10", "Unlinked");
        db.execute_batch(
            "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES ('a', 'A', 0, 0, 0), ('b', 'B', 0, 0, 0);",
        )
        .unwrap();
        for tag_id in ["a", "b"] {
            db.execute(
                "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
                (&linked, tag_id),
            )
            .unwrap();
        }
        for (id, number) in [("l1", 1), ("l2", 2)] {
            db.execute(
                "INSERT INTO todo_links (id, todo_id, kind, url, github_owner, github_repo, github_number, created_at)
                 VALUES (?1, ?2, 'github_issue', ?1, 'o', 'r', ?3, 0)",
                (id, &linked, number),
            )
            .unwrap();
        }

        let todos = query_todos(&db, &day("2026-10-10"), &day("2026-10-10")).unwrap();
        assert_eq!(todos.len(), 2);
        let todo = |id: &str| todos.iter().find(|todo| todo.id == id).unwrap();
        assert_eq!(todo(&linked).tag_ids, ["a", "b"]);
        let labels: Vec<&str> = todo(&linked)
            .links
            .iter()
            .map(|link| link.label.as_str())
            .collect();
        assert_eq!(labels, ["r#1", "r#2"]);
        assert!(todo(&unlinked).links.is_empty());
    }

    #[test]
    fn create_todo_from_link_creates_it_with_its_link_at_the_end_of_the_day() {
        let mut db = open_test_database(&WORKSPACE);
        let first = insert_todo(&db, "2026-10-10", "First");
        let link = links::parse("https://github.com/pierreyoda/todai/pull/58/files").unwrap();

        let todo = create_with_link(
            &mut db,
            &day("2026-10-10"),
            "  Add todo_links table and link chips ",
            link,
            NOW,
        )
        .unwrap();
        assert_eq!(todo.title, "Add todo_links table and link chips");
        assert!(todo.position > todo_row(&db, &first).position);
        assert_eq!(todo.links.len(), 1);
        assert_eq!(
            todo.links[0].url,
            "https://github.com/pierreyoda/todai/pull/58"
        );
        let listed = query_todos(&db, &day("2026-10-10"), &day("2026-10-10")).unwrap();
        assert_eq!(listed[1].links[0].label, "todai#58");
    }

    #[test]
    fn create_todo_from_link_creates_nothing_without_a_title() {
        let mut db = open_test_database(&WORKSPACE);
        let link = links::parse("https://example.org").unwrap();
        assert!(create_with_link(&mut db, &day("2026-10-10"), " ", link, NOW).is_err());
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM todos", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
