use jiff::{civil::Date, tz::TimeZone, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::{DbColor, DbDay, DbNote, DbTag, DbTodo},
    errors::{Result, TodaiError},
    models::integrations::TodoLink,
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
    /// Its links, in the order they were added: unlike tags, each belongs to this todo only.
    pub links: Vec<TodoLink>,
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
            links: Vec::new(),
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

/// A tag's color.
///
/// (De)serialized as a `#RRGGBB` string, matching the frontend's `Tag.color`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "String", into = "String")]
pub struct Color {
    pub raw: DbColor,
    /// Format: #RRGGBB (uppercase).
    pub rgb: String,
}

impl TryFrom<DbColor> for Color {
    type Error = TodaiError;

    fn try_from(raw: DbColor) -> std::result::Result<Self, Self::Error> {
        if raw > 0xFFFFFF {
            return Err(TodaiError::InvalidColor(format!("{raw:#X}")));
        }
        Ok(Self {
            raw,
            rgb: format!("#{raw:06X}"),
        })
    }
}

impl From<Color> for DbColor {
    fn from(color: Color) -> Self {
        color.raw
    }
}

impl TryFrom<String> for Color {
    type Error = TodaiError;

    fn try_from(rgb: String) -> std::result::Result<Self, Self::Error> {
        let raw = rgb
            .strip_prefix('#')
            .filter(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
            .and_then(|hex| DbColor::from_str_radix(hex, 16).ok())
            .ok_or(TodaiError::InvalidColor(rgb))?;
        raw.try_into()
    }
}

impl From<Color> for String {
    fn from(color: Color) -> Self {
        color.rgb
    }
}

/// A tag, categorizing todos.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Color,
    /// Number of non-deleted todos with this tag. Only set by `list_tags`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_todos_count: Option<u32>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}

impl TryFrom<DbTag> for Tag {
    type Error = TodaiError;

    fn try_from(tag: DbTag) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: tag.id,
            name: tag.name,
            color: tag.color.try_into()?,
            linked_todos_count: None,
            created_at: Timestamp::from_second(tag.created_at)?,
            updated_at: Timestamp::from_second(tag.updated_at)?,
            deleted_at: tag.deleted_at.map(Timestamp::from_second).transpose()?,
        })
    }
}

/// A day's note, in Markdown.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub day: Day,
    /// Never blank: a note emptied is deleted.
    pub content: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl TryFrom<DbNote> for Note {
    type Error = TodaiError;

    fn try_from(note: DbNote) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            day: note.day.try_into()?,
            content: note.content,
            created_at: Timestamp::from_second(note.created_at)?,
            updated_at: Timestamp::from_second(note.updated_at)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        database::models::DbTodo,
        models::core::{Estimate, Todo},
    };

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
