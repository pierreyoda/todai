use fractional_index::FractionalIndex;
use jiff::{civil::Date, tz::TimeZone, Timestamp};
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::{
    database::models::{DbDay, DbTodo},
    errors::{Result, TodaiError},
    state::AppState,
};

/// A todo's day.
///
/// Deserialized (e.g. as a command argument) from a strict `YYYY-MM-DD` string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", try_from = "DbDay")]
pub struct Day {
    /// Format: YYYY-MM-DD
    pub raw: DbDay,
    /// Midnight UTC of `raw`.
    pub timestamp: Timestamp,
}

impl TryFrom<DbDay> for Day {
    type Error = TodaiError;

    fn try_from(raw: DbDay) -> std::result::Result<Self, Self::Error> {
        let date: Date = raw.parse()?;
        // jiff also accepts e.g. `20261001` or `2026-10-01T10:00`: only keep the canonical form.
        if date.to_string() != raw {
            return Err(TodaiError::InvalidDay(raw));
        }
        let timestamp = date.to_zoned(TimeZone::UTC)?.timestamp();
        Ok(Self { raw, timestamp })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub day: Day,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub position: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub deleted_at: Option<Timestamp>,
}

impl TryFrom<DbTodo> for Todo {
    type Error = TodaiError;

    fn try_from(todo: DbTodo) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: todo.id,
            day: todo.day.try_into()?,
            title: todo.title,
            description: todo.description,
            completed: todo.completed == 1,
            position: todo.position,
            created_at: Timestamp::from_second(todo.created_at)?,
            updated_at: Timestamp::from_second(todo.updated_at)?,
            completed_at: todo.completed_at.map(Timestamp::from_second).transpose()?,
            deleted_at: todo.deleted_at.map(Timestamp::from_second).transpose()?,
        })
    }
}

/// Lists the non-deleted todos of `day`, in display order.
#[tauri::command]
pub async fn list_todos(state: State<'_, AppState>, day: Day) -> Result<Vec<Todo>> {
    let db = state.db();
    let mut statement = db.prepare_cached(
        "SELECT * FROM todos WHERE day = ?1 AND deleted_at IS NULL ORDER BY position",
    )?;
    let todos = statement
        .query_map([&day.raw], DbTodo::from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    todos.into_iter().map(Todo::try_from).collect()
}

/// Creates a todo at the end of `day`.
///
/// `title` and `description` are trimmed; an empty `description` is stored as `NULL`.
#[tauri::command]
pub async fn create_todo(
    state: State<'_, AppState>,
    day: Day,
    title: String,
    description: Option<String>,
) -> Result<Todo> {
    let title = title.trim();
    if title.is_empty() {
        return Err(TodaiError::CommandError(
            "Todo title cannot be empty".into(),
        ));
    }
    let description = description
        .as_deref()
        .map(str::trim)
        .filter(|description| !description.is_empty());

    let db = state.db();
    // Includes deleted todos, so that restoring one never collides with a newer position.
    let last_position: Option<String> = db
        .prepare_cached("SELECT MAX(position) FROM todos WHERE day = ?1")?
        .query_row([&day.raw], |row| row.get(0))?;
    let position = match last_position {
        Some(last) => FractionalIndex::new_after(
            &FractionalIndex::from_string(&last).map_err(|_| TodaiError::InvalidPosition(last))?,
        ),
        None => FractionalIndex::default(),
    };

    let now = Timestamp::now().as_second();
    let todo = db
        .prepare_cached(
            "INSERT INTO todos (id, day, title, description, position, created_at, updated_at)
             VALUES (:id, :day, :title, :description, :position, :now, :now)
             RETURNING *",
        )?
        .query_row(
            named_params! {
                ":id": Uuid::now_v7().to_string(),
                ":day": day.raw,
                ":title": title,
                ":description": description,
                ":position": position.to_string(),
                ":now": now,
            },
            DbTodo::from_row,
        )?;
    todo.try_into()
}
