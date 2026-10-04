use fractional_index::FractionalIndex;
use jiff::{civil::Date, tz::TimeZone, Timestamp};
use rusqlite::{named_params, Connection};
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
/// (De)serialized as a strict `YYYY-MM-DD` string, matching the frontend's `Day`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "DbDay", into = "DbDay")]
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

impl From<Day> for DbDay {
    fn from(day: Day) -> Self {
        day.raw
    }
}

/// A todo's estimate: either a duration or story points.
///
/// (De)serialized as `{ "unit": "minutes" | "points", "value": <positive integer> }`, matching the frontend's `Estimate`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "unit", content = "value", rename_all = "lowercase")]
pub enum Estimate {
    Minutes(u32),
    Points(u32),
}

impl Estimate {
    /// A day.
    pub const MAX_MINUTES: u32 = 24 * 60;
    pub const MAX_POINTS: u32 = 100;

    const DB_MINUTES: &str = "minutes";
    const DB_POINTS: &str = "points";

    /// Checks that the value is positive and within its unit's maximum.
    ///
    /// Not enforced when reading from the database, so that lowering a maximum never makes todos unreadable.
    pub fn validate(self) -> Result<Self> {
        let (value, max) = match self {
            Self::Minutes(minutes) => (minutes, Self::MAX_MINUTES),
            Self::Points(points) => (points, Self::MAX_POINTS),
        };
        if value == 0 || value > max {
            return Err(TodaiError::InvalidEstimate(format!(
                "{self:?} is not between 1 and {max}"
            )));
        }
        Ok(self)
    }

    /// The `estimate` and `estimate_unit` columns.
    pub fn to_db(self) -> (u32, &'static str) {
        match self {
            Self::Minutes(minutes) => (minutes, Self::DB_MINUTES),
            Self::Points(points) => (points, Self::DB_POINTS),
        }
    }
}

/// From the `estimate` and `estimate_unit` columns, when set.
impl TryFrom<(u32, &str)> for Estimate {
    type Error = TodaiError;

    fn try_from((value, unit): (u32, &str)) -> std::result::Result<Self, Self::Error> {
        match unit {
            Self::DB_MINUTES => Ok(Self::Minutes(value)),
            Self::DB_POINTS => Ok(Self::Points(value)),
            _ => Err(TodaiError::InvalidEstimate(format!(
                "unknown unit {unit:?}"
            ))),
        }
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
    pub estimate: Option<Estimate>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub deleted_at: Option<Timestamp>,
    /// IDs of its non-deleted tags, to resolve against `list_tags`: tags have a single source of truth on the
    /// frontend, so that renaming one doesn't require refetching todos.
    pub tag_ids: Vec<String>,
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
            // The columns are either both set, or both `NULL` for no estimate.
            estimate: match (todo.estimate, todo.estimate_unit.as_deref()) {
                (None, None) => None,
                (Some(value), Some(unit)) => Some((value, unit).try_into()?),
                (value, unit) => {
                    return Err(TodaiError::InvalidEstimate(format!(
                        "unexpected value {value:?} with unit {unit:?}"
                    )))
                }
            },
            position: todo.position,
            created_at: Timestamp::from_second(todo.created_at)?,
            updated_at: Timestamp::from_second(todo.updated_at)?,
            completed_at: todo.completed_at.map(Timestamp::from_second).transpose()?,
            deleted_at: todo.deleted_at.map(Timestamp::from_second).transpose()?,
            tag_ids: Vec::new(),
        })
    }
}

/// A month's statistics about its non-deleted todos.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TodoMonth {
    /// Format: YYYY-MM
    pub month: String,
    pub count: u32,
    pub completed_count: u32,
    /// Sum of the estimates in minutes.
    pub estimated_minutes: u32,
    /// Sum of the estimates in story points.
    pub estimated_points: u32,
}

/// Lists the non-deleted todos of `day`, in display order, with the IDs of their non-deleted tags.
#[tauri::command]
pub async fn list_todos(state: State<'_, AppState>, day: Day) -> Result<Vec<Todo>> {
    query_todos(&state.db(), &day, &day)
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
    query_todos(&state.db(), &start, &end)
}

/// Lists the months having non-deleted todos, most recent first, with their statistics.
#[tauri::command]
pub async fn list_todo_months(state: State<'_, AppState>) -> Result<Vec<TodoMonth>> {
    let db = state.db();
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

/// The non-deleted todos from `start` to `end` (inclusive), by day then in display order.
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
    Ok(todos)
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
    let (estimate, estimate_unit) = estimate
        .map(Estimate::validate)
        .transpose()?
        .map(Estimate::to_db)
        .unzip();

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

/// Updates a non-deleted todo's `title`, trimmed and not empty, as in `create_todo`.
#[tauri::command]
pub async fn update_todo(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
) -> Result<()> {
    let title = title.as_deref().map(str::trim);
    if title.is_some_and(str::is_empty) {
        return Err(TodaiError::CommandError(
            "Todo title cannot be empty".into(),
        ));
    }

    let db = state.db();
    let now = Timestamp::now().as_second();
    let updated = db
        .prepare_cached(
            "UPDATE todos
                 SET title = COALESCE(:title, title),
                     updated_at = :now
                 WHERE id = :id",
        )?
        .execute(named_params! { ":id": id, ":title": title, ":now": now })?;
    if updated == 0 {
        return Err(TodaiError::CommandError(format!("Todo {id} not found")));
    }
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

    let db = state.db();
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
    let db = state.db();
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
    let db = state.db();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_serializes_with_its_unit() {
        let json = |estimate: Estimate| serde_json::to_value(estimate).unwrap();
        assert_eq!(
            json(Estimate::Minutes(90)),
            serde_json::json!({ "unit": "minutes", "value": 90 })
        );
        assert_eq!(
            json(Estimate::Points(3)),
            serde_json::json!({ "unit": "points", "value": 3 })
        );
    }

    #[test]
    fn estimate_deserializes_known_units_only() {
        let parse = |json: &str| serde_json::from_str::<Estimate>(json);
        assert_eq!(
            parse(r#"{ "unit": "minutes", "value": 30 }"#).unwrap(),
            Estimate::Minutes(30)
        );
        assert_eq!(
            parse(r#"{ "unit": "points", "value": 5 }"#).unwrap(),
            Estimate::Points(5)
        );
        assert!(parse(r#"{ "unit": "hours", "value": 1 }"#).is_err());
        assert!(parse(r#"{ "unit": "points", "value": -1 }"#).is_err());
        assert!(parse(r#"{ "unit": "points", "value": 1.5 }"#).is_err());
        assert!(parse(r#"{ "value": 1 }"#).is_err());
    }

    #[test]
    fn estimate_validates_its_bounds() {
        for valid in [
            Estimate::Minutes(1),
            Estimate::Minutes(Estimate::MAX_MINUTES),
            Estimate::Points(1),
            Estimate::Points(Estimate::MAX_POINTS),
        ] {
            assert_eq!(valid.validate().unwrap(), valid);
        }
        for invalid in [
            Estimate::Minutes(0),
            Estimate::Minutes(Estimate::MAX_MINUTES + 1),
            Estimate::Points(0),
            Estimate::Points(Estimate::MAX_POINTS + 1),
        ] {
            assert!(invalid.validate().is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn estimate_round_trips_through_the_database_columns() {
        for estimate in [Estimate::Minutes(45), Estimate::Points(8)] {
            assert_eq!(Estimate::try_from(estimate.to_db()).unwrap(), estimate);
        }
    }

    #[test]
    fn estimate_rejects_unknown_database_units() {
        assert!(Estimate::try_from((3, "hours")).is_err());
        assert!(Estimate::try_from((3, "Minutes")).is_err());
    }

    fn db_todo(estimate: Option<u32>, estimate_unit: Option<&str>) -> DbTodo {
        DbTodo {
            id: "019a0000-0000-7000-8000-000000000000".into(),
            day: "2026-10-04".into(),
            title: "Title".into(),
            description: None,
            completed: 0,
            position: "80".into(),
            estimate,
            estimate_unit: estimate_unit.map(Into::into),
            created_at: 0,
            updated_at: 0,
            completed_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn todo_reads_its_estimate_from_both_columns() {
        let estimate = |db_todo| Todo::try_from(db_todo).unwrap().estimate;
        assert_eq!(estimate(db_todo(None, None)), None);
        assert_eq!(
            estimate(db_todo(Some(30), Some("minutes"))),
            Some(Estimate::Minutes(30))
        );
        assert_eq!(
            estimate(db_todo(Some(3), Some("points"))),
            Some(Estimate::Points(3))
        );
    }

    #[test]
    fn todo_rejects_inconsistent_estimate_columns() {
        assert!(Todo::try_from(db_todo(Some(3), None)).is_err());
        assert!(Todo::try_from(db_todo(None, Some("points"))).is_err());
        assert!(Todo::try_from(db_todo(Some(3), Some("hours"))).is_err());
    }
}
