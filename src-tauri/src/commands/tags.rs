use std::collections::HashSet;

use jiff::Timestamp;
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::{
    database::models::{DbColor, DbTag},
    errors::{Result, TodaiError},
    state::AppState,
};

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

/// Trims `name`, which cannot be empty.
fn validate_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(TodaiError::CommandError("Tag name cannot be empty".into()));
    }
    Ok(name)
}

fn tag_not_found(id: &str) -> TodaiError {
    TodaiError::CommandError(format!("Tag {id} not found"))
}

/// Lists the non-deleted tags, sorted by name, with their number of linked todos.
#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>> {
    let db = state.db()?;
    // Counted per tag through `idx_todo_tags_tag`; deleted todos keep their links, so they are excluded here.
    let mut statement = db.prepare_cached(
        "SELECT tags.*,
                (SELECT COUNT(*)
                     FROM todo_tags
                     JOIN todos ON todos.id = todo_tags.todo_id
                     WHERE todo_tags.tag_id = tags.id AND todos.deleted_at IS NULL
                ) AS linked_todos_count
         FROM tags
         WHERE deleted_at IS NULL
         ORDER BY name",
    )?;
    let rows = statement
        .query_and_then([], |row| {
            Ok((DbTag::from_row(row)?, row.get("linked_todos_count")?))
        })?
        .collect::<Result<Vec<(DbTag, u32)>>>()?;
    rows.into_iter()
        .map(|(tag, linked_todos_count)| {
            Ok(Tag {
                linked_todos_count: Some(linked_todos_count),
                ..tag.try_into()?
            })
        })
        .collect()
}

/// Creates a tag. `name` is trimmed, and cannot be empty.
#[tauri::command]
pub async fn create_tag(state: State<'_, AppState>, name: String, color: Color) -> Result<Tag> {
    let name = validate_name(&name)?;

    let db = state.db()?;
    let now = Timestamp::now().as_second();
    let tag = db
        .prepare_cached(
            "INSERT INTO tags (id, name, color, created_at, updated_at)
             VALUES (:id, :name, :color, :now, :now)
             RETURNING *",
        )?
        .query_and_then(
            named_params! {
                ":id": Uuid::now_v7().to_string(),
                ":name": name,
                ":color": DbColor::from(color),
                ":now": now,
            },
            DbTag::from_row,
        )?
        .next()
        .transpose()?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    tag.try_into()
}

/// Updates a non-deleted tag's `name` and/or `color`, with the same rules as `create_tag`.
#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    color: Option<Color>,
) -> Result<Tag> {
    let name = name.as_deref().map(validate_name).transpose()?;

    let db = state.db()?;
    let now = Timestamp::now().as_second();
    let tag = db
        .prepare_cached(
            "UPDATE tags
                 SET name = COALESCE(:name, name),
                     color = COALESCE(:color, color),
                     updated_at = :now
                 WHERE id = :id AND deleted_at IS NULL
                 RETURNING *",
        )?
        .query_and_then(
            named_params! {
                ":id": id,
                ":name": name,
                ":color": color.map(DbColor::from),
                ":now": now,
            },
            DbTag::from_row,
        )?
        .next()
        .transpose()?
        .ok_or_else(|| tag_not_found(&id))?;
    tag.try_into()
}

/// Deletes a tag, keeping its row and links to todos so that it can be restored.
#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, id: String) -> Result<()> {
    let db = state.db()?;
    let now = Timestamp::now().as_second();
    let deleted = db
        .prepare_cached(
            "UPDATE tags
                 SET deleted_at = :now,
                     updated_at = :now
                 WHERE id = :id AND deleted_at IS NULL",
        )?
        .execute(named_params! { ":id": id, ":now": now })?;
    if deleted == 0 {
        return Err(tag_not_found(&id));
    }
    Ok(())
}

/// Replaces the tags of a non-deleted todo with `tag_ids`, which must all be non-deleted tags.
///
/// Links to deleted tags are kept, so that restoring a tag restores it on its todos.
#[tauri::command]
pub async fn set_todo_tags(
    state: State<'_, AppState>,
    todo_id: String,
    tag_ids: Vec<String>,
) -> Result<()> {
    let tag_ids: HashSet<String> = tag_ids.into_iter().collect();

    let mut db = state.db()?;
    let tx = db.transaction()?;
    let now = Timestamp::now().as_second();
    let touched = tx
        .prepare_cached("UPDATE todos SET updated_at = :now WHERE id = :id AND deleted_at IS NULL")?
        .execute(named_params! { ":id": todo_id, ":now": now })?;
    if touched == 0 {
        return Err(TodaiError::CommandError(format!(
            "Todo {todo_id} not found"
        )));
    }

    tx.prepare_cached(
        "DELETE FROM todo_tags
         WHERE todo_id = ?1 AND tag_id IN (SELECT id FROM tags WHERE deleted_at IS NULL)",
    )?
    .execute([&todo_id])?;
    {
        let mut insert = tx.prepare_cached(
            "INSERT INTO todo_tags (todo_id, tag_id)
             SELECT ?1, id FROM tags WHERE id = ?2 AND deleted_at IS NULL",
        )?;
        for tag_id in &tag_ids {
            if insert.execute([&todo_id, tag_id])? == 0 {
                return Err(tag_not_found(tag_id));
            }
        }
    }
    tx.commit()?;
    Ok(())
}
