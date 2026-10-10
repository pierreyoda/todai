use jiff::Timestamp;
use rusqlite::{named_params, Connection, OptionalExtension};
use serde::Serialize;
use tauri::State;

use crate::{
    commands::todos::Day,
    database::models::DbNote,
    errors::{Result, TodaiError},
    state::AppState,
};

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

/// The note of `day`, if it has one.
#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, day: Day) -> Result<Option<Note>> {
    let db = state.db()?;
    query_note(&db, &day)
}

/// Saves `content` as the note of `day`, as typed (not trimmed), and returns it. A blank `content` deletes the note
/// instead, returning `None`: days without a note have no row.
///
/// Saving the same content again leaves its `updated_at` unchanged.
#[tauri::command]
pub async fn save_note(
    state: State<'_, AppState>,
    day: Day,
    content: String,
) -> Result<Option<Note>> {
    let db = state.db()?;
    save_note_row(&db, &day, &content, Timestamp::now().as_second())
}

fn query_note(db: &Connection, day: &Day) -> Result<Option<Note>> {
    db.prepare_cached("SELECT * FROM notes WHERE day = ?1")?
        .query_row([&day.raw], DbNote::from_row)
        .optional()?
        .map(Note::try_from)
        .transpose()
}

fn save_note_row(db: &Connection, day: &Day, content: &str, now: i64) -> Result<Option<Note>> {
    if content.trim().is_empty() {
        db.prepare_cached("DELETE FROM notes WHERE day = ?1")?
            .execute([&day.raw])?;
        return Ok(None);
    }
    // Read back rather than `RETURNING`, which returns nothing when the content is unchanged.
    db.prepare_cached(
        "INSERT INTO notes (day, content, created_at, updated_at)
         VALUES (:day, :content, :now, :now)
         ON CONFLICT (day) DO UPDATE
             SET content = excluded.content,
                 updated_at = excluded.updated_at
             WHERE notes.content <> excluded.content",
    )?
    .execute(named_params! {
        ":day": day.raw,
        ":content": content,
        ":now": now,
    })?;
    query_note(db, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{open_test_database, WORKSPACE};

    const NOW: i64 = 1_791_000_000;

    fn day(raw: &str) -> Day {
        raw.to_string().try_into().unwrap()
    }

    fn save(db: &Connection, day_raw: &str, content: &str, now: i64) -> Option<Note> {
        save_note_row(db, &day(day_raw), content, now).unwrap()
    }

    fn get(db: &Connection, day_raw: &str) -> Option<Note> {
        query_note(db, &day(day_raw)).unwrap()
    }

    fn notes_count(db: &Connection) -> i64 {
        db.query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn a_day_has_no_note_until_one_is_saved() {
        let db = open_test_database(&WORKSPACE);
        assert!(get(&db, "2026-10-10").is_none());

        let saved = save(&db, "2026-10-10", "# Saturday", NOW).unwrap();
        assert_eq!(saved.day, day("2026-10-10"));
        assert_eq!(saved.content, "# Saturday");
        assert_eq!(saved.created_at.as_second(), NOW);
        assert_eq!(saved.updated_at.as_second(), NOW);
        assert_eq!(get(&db, "2026-10-10").unwrap().content, "# Saturday");
    }

    #[test]
    fn save_note_updates_the_content_and_keeps_created_at() {
        let db = open_test_database(&WORKSPACE);
        save(&db, "2026-10-10", "Before", NOW);
        let saved = save(&db, "2026-10-10", "After", NOW + 60).unwrap();
        assert_eq!(saved.content, "After");
        assert_eq!(saved.created_at.as_second(), NOW);
        assert_eq!(saved.updated_at.as_second(), NOW + 60);
        assert_eq!(notes_count(&db), 1);
    }

    #[test]
    fn save_note_keeps_updated_at_when_the_content_is_unchanged() {
        let db = open_test_database(&WORKSPACE);
        save(&db, "2026-10-10", "Same", NOW);
        let saved = save(&db, "2026-10-10", "Same", NOW + 60).unwrap();
        assert_eq!(saved.content, "Same");
        assert_eq!(saved.updated_at.as_second(), NOW);
    }

    #[test]
    fn save_note_keeps_the_content_as_typed() {
        let db = open_test_database(&WORKSPACE);
        let content = "  - [ ] Indented task\n\nTrailing line\n\n";
        assert_eq!(
            save(&db, "2026-10-10", content, NOW).unwrap().content,
            content
        );
    }

    #[test]
    fn save_note_deletes_the_note_when_blank() {
        let db = open_test_database(&WORKSPACE);
        save(&db, "2026-10-10", "# Saturday", NOW);
        assert!(save(&db, "2026-10-10", " \n\t\n", NOW + 60).is_none());
        assert!(get(&db, "2026-10-10").is_none());
        assert_eq!(notes_count(&db), 0);

        // Without a note to delete.
        assert!(save(&db, "2026-10-11", "", NOW).is_none());
        assert_eq!(notes_count(&db), 0);

        // Written again: a new note.
        let saved = save(&db, "2026-10-10", "Again", NOW + 120).unwrap();
        assert_eq!(saved.created_at.as_second(), NOW + 120);
    }

    #[test]
    fn notes_are_kept_per_day() {
        let db = open_test_database(&WORKSPACE);
        save(&db, "2026-10-09", "Friday", NOW);
        save(&db, "2026-10-10", "Saturday", NOW);
        save(&db, "2026-10-10", "", NOW + 60);
        assert_eq!(get(&db, "2026-10-09").unwrap().content, "Friday");
        assert!(get(&db, "2026-10-10").is_none());
    }

    #[test]
    fn the_database_rejects_empty_notes() {
        let db = open_test_database(&WORKSPACE);
        assert!(db
            .execute(
                "INSERT INTO notes (day, content, created_at, updated_at) VALUES ('2026-10-10', '', 0, 0)",
                [],
            )
            .is_err());
    }

    #[test]
    fn note_serializes_with_its_day_and_timestamps() {
        let db = open_test_database(&WORKSPACE);
        let note = save(&db, "2026-10-10", "# Saturday", NOW).unwrap();
        assert_eq!(
            serde_json::to_value(note).unwrap(),
            serde_json::json!({
                "day": "2026-10-10",
                "content": "# Saturday",
                "createdAt": "2026-10-03T04:00:00Z",
                "updatedAt": "2026-10-03T04:00:00Z",
            })
        );
    }
}
