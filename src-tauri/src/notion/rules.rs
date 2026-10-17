//! Which values of which column of a Notion database mean its pages are done, when Notion's own Status groups don't
//! tell it as the user means it: e.g. a Stage select whose Released and Won't do options are done.

use std::collections::HashMap;

use jiff::Timestamp;
use rusqlite::{named_params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::snapshot::{
    ColumnKind, NotionColumnValue, NotionOptionValue, NotionPageSnapshot, NotionStatus, StatusGroup,
};
use crate::{
    database::models::DbTimestamp,
    errors::{Result, TodaiError},
    links::snapshot::LinkSnapshot,
    models::integrations::TodoLink,
};

/// An option meaning done, by its ID, with its name when the rule was set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DoneOption {
    pub id: String,
    pub name: String,
}

/// A database's rule: its pages are done once its column `property_id` holds one of `done_options` (or is checked).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotionDoneRule {
    pub data_source_id: String,
    pub database_title: String,
    pub property_id: String,
    pub property_name: String,
    pub property_kind: ColumnKind,
    /// Empty for a checkbox.
    pub done_options: Vec<DoneOption>,
    #[serde(default)]
    pub updated_at: Option<Timestamp>,
}

/// The rules of the workspace, by data source ID.
pub type Rules = HashMap<String, NotionDoneRule>;

fn invalid(reason: impl Into<String>) -> TodaiError {
    TodaiError::CommandError(reason.into())
}

fn from_row(row: &Row) -> Result<NotionDoneRule> {
    let kind: String = row.get("property_kind")?;
    let done_options: String = row.get("done_options")?;
    let updated_at: DbTimestamp = row.get("updated_at")?;
    Ok(NotionDoneRule {
        data_source_id: row.get("data_source_id")?,
        database_title: row.get("database_title")?,
        property_id: row.get("property_id")?,
        property_name: row.get("property_name")?,
        property_kind: ColumnKind::from_db(&kind)
            .ok_or_else(|| invalid(format!("Unknown column kind {kind:?}")))?,
        done_options: serde_json::from_str(&done_options)?,
        updated_at: Some(Timestamp::from_second(updated_at)?),
    })
}

/// The rules of the workspace database `db`, by their database's title.
pub fn list(db: &Connection) -> Result<Vec<NotionDoneRule>> {
    let mut statement = db.prepare_cached(
        "SELECT * FROM notion_done_rules ORDER BY database_title COLLATE NOCASE, data_source_id",
    )?;
    let rows = statement.query_map([], |row| Ok(from_row(row)))?;
    rows.map(|row| row?).collect()
}

/// The rules of `db`, to apply.
pub fn load(db: &Connection) -> Result<Rules> {
    Ok(list(db)?
        .into_iter()
        .map(|rule| (rule.data_source_id.clone(), rule))
        .collect())
}

/// Sets the rule of its database, instead of any other one, at `now`.
pub fn set(db: &Connection, rule: &NotionDoneRule, now: DbTimestamp) -> Result<NotionDoneRule> {
    if rule.data_source_id.is_empty() || rule.property_id.is_empty() {
        return Err(invalid("A done rule needs a database and a column"));
    }
    match (rule.property_kind, rule.done_options.is_empty()) {
        (ColumnKind::Checkbox, false) => {
            return Err(invalid(
                "A checkbox's rule has no options: it's done when checked",
            ))
        }
        (ColumnKind::Checkbox, true) | (_, false) => {}
        (_, true) => return Err(invalid("Pick at least one value meaning done")),
    }
    db.prepare_cached(
        "INSERT INTO notion_done_rules
         (data_source_id, database_title, property_id, property_name, property_kind, done_options, updated_at)
         VALUES (:data_source_id, :database_title, :property_id, :property_name, :property_kind, :done_options, :now)
         ON CONFLICT (data_source_id) DO UPDATE SET
           database_title = excluded.database_title, property_id = excluded.property_id,
           property_name = excluded.property_name, property_kind = excluded.property_kind,
           done_options = excluded.done_options, updated_at = excluded.updated_at
         RETURNING *",
    )?
    .query_row(
        named_params! {
            ":data_source_id": rule.data_source_id,
            ":database_title": rule.database_title,
            ":property_id": rule.property_id,
            ":property_name": rule.property_name,
            ":property_kind": rule.property_kind.to_db(),
            ":done_options": serde_json::to_string(&rule.done_options)?,
            ":now": now,
        },
        |row| Ok(from_row(row)),
    )?
}

/// Deletes the rule of the database `data_source_id`, returning whether there was one: its pages are then done once
/// their Status is in its Complete group again.
pub fn delete(db: &Connection, data_source_id: &str) -> Result<bool> {
    Ok(db
        .prepare_cached("DELETE FROM notion_done_rules WHERE data_source_id = ?1 RETURNING 1")?
        .query_row([data_source_id], |_| Ok(()))
        .optional()?
        .is_some())
}

/// Shows the Notion page of `link`, if any, as its database's rule tells: see `apply`.
pub fn apply_to_link(link: &mut TodoLink, rules: &Rules) {
    if let Some(snapshot) = &mut link.snapshot {
        apply_to_snapshot(snapshot, rules);
    }
}

/// Shows `snapshot`, if a Notion page's, as its database's rule tells: see `apply`.
pub fn apply_to_snapshot(snapshot: &mut LinkSnapshot, rules: &Rules) {
    if let LinkSnapshot::NotionPage(page) = snapshot {
        if let Some(rule) = page
            .database
            .as_ref()
            .and_then(|database| rules.get(&database.data_source_id))
        {
            apply(page, rule);
        }
    }
}

/// Shows `page` as `rule` tells: its status becomes the value of the rule's column, in the Complete group when it's
/// one meaning done. Unchanged if the page doesn't have that column (e.g. deleted, or a snapshot stored before columns
/// were kept), so that it's done as Notion's Status groups tell instead.
pub fn apply(page: &mut NotionPageSnapshot, rule: &NotionDoneRule) {
    let Some(column) = page
        .columns
        .iter()
        .find(|column| column.property_id == rule.property_id)
        .filter(|column| column.value.kind() == rule.property_kind)
    else {
        return;
    };
    let is_done =
        |option: &NotionOptionValue| rule.done_options.iter().any(|done| done.id == option.id);
    let status = match &column.value {
        NotionColumnValue::Status { option } | NotionColumnValue::Select { option } => {
            option.as_ref().map(|option| {
                let done = is_done(option);
                // Its own group, for the page's Status: unless it's a Complete one the rule doesn't count as done
                let group = match &page.status {
                    Some(status) if status.name == option.name && !done => {
                        status.group.filter(|group| *group != StatusGroup::Complete)
                    }
                    _ => None,
                };
                NotionStatus {
                    name: option.name.clone(),
                    color: option.color.clone(),
                    group: if done {
                        Some(StatusGroup::Complete)
                    } else {
                        group
                    },
                }
            })
        }
        NotionColumnValue::MultiSelect { options } => {
            let done: Vec<&NotionOptionValue> =
                options.iter().filter(|option| is_done(option)).collect();
            let shown = if done.is_empty() {
                options.iter().collect()
            } else {
                done.clone()
            };
            shown.first().map(|first| NotionStatus {
                name: shown
                    .iter()
                    .map(|option| option.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                color: first.color.clone(),
                group: (!done.is_empty()).then_some(StatusGroup::Complete),
            })
        }
        NotionColumnValue::Checkbox { checked } => checked.then(|| NotionStatus {
            name: column.name.clone(),
            color: "green".into(),
            group: Some(StatusGroup::Complete),
        }),
    };
    page.status = status;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::{open_test_database, WORKSPACE},
        notion::snapshot::{NotionColumn, NotionParentDatabase},
    };

    const NOW: i64 = 1_791_000_000;

    fn option(id: &str, name: &str, color: &str) -> NotionOptionValue {
        NotionOptionValue {
            id: id.into(),
            name: name.into(),
            color: color.into(),
        }
    }

    fn rule(kind: ColumnKind, property_id: &str, done: &[(&str, &str)]) -> NotionDoneRule {
        NotionDoneRule {
            data_source_id: "ds1".into(),
            database_title: "Tickets".into(),
            property_id: property_id.into(),
            property_name: "Stage".into(),
            property_kind: kind,
            done_options: done
                .iter()
                .map(|(id, name)| DoneOption {
                    id: (*id).into(),
                    name: (*name).into(),
                })
                .collect(),
            updated_at: None,
        }
    }

    fn page(columns: Vec<NotionColumn>) -> NotionPageSnapshot {
        NotionPageSnapshot {
            title: "Ticket".into(),
            icon: None,
            unique_id: None,
            status: Some(NotionStatus {
                name: "In progress".into(),
                color: "blue".into(),
                group: Some(StatusGroup::InProgress),
            }),
            assignees: Vec::new(),
            due: None,
            tags: Vec::new(),
            last_edited_at: Timestamp::from_second(NOW).unwrap(),
            in_trash: false,
            database: Some(NotionParentDatabase {
                data_source_id: "ds1".into(),
                title: Some("Tickets".into()),
            }),
            columns,
        }
    }

    fn stage(value: Option<NotionOptionValue>) -> NotionColumn {
        NotionColumn {
            property_id: "st".into(),
            name: "Stage".into(),
            value: NotionColumnValue::Select { option: value },
        }
    }

    #[test]
    fn rules_are_kept_by_database() {
        let db = open_test_database(&WORKSPACE);
        let released = rule(ColumnKind::Select, "st", &[("opt-released", "Released")]);
        let kept = set(&db, &released, NOW).unwrap();
        assert_eq!(kept.done_options, released.done_options);
        assert_eq!(kept.updated_at, Some(Timestamp::from_second(NOW).unwrap()));

        // Replaced, not added
        let wider = rule(
            ColumnKind::Select,
            "st",
            &[("opt-released", "Released"), ("opt-wontdo", "Won't do")],
        );
        set(&db, &wider, NOW + 1).unwrap();
        let rules = list(&db).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].done_options.len(), 2);
        assert!(load(&db).unwrap().contains_key("ds1"));

        assert!(delete(&db, "ds1").unwrap());
        assert!(!delete(&db, "ds1").unwrap());
        assert!(list(&db).unwrap().is_empty());
    }

    #[test]
    fn a_rule_needs_values_unless_it_s_a_checkbox() {
        let db = open_test_database(&WORKSPACE);
        assert!(set(&db, &rule(ColumnKind::Select, "st", &[]), NOW).is_err());
        assert!(set(&db, &rule(ColumnKind::Checkbox, "sh", &[("x", "X")]), NOW).is_err());
        assert!(set(&db, &rule(ColumnKind::Checkbox, "sh", &[]), NOW).is_ok());
        assert!(set(&db, &rule(ColumnKind::Select, "", &[("x", "X")]), NOW).is_err());
    }

    #[test]
    fn a_select_rule_shows_its_column_and_tells_when_done() {
        let released = rule(ColumnKind::Select, "st", &[("opt-released", "Released")]);
        let mut qa = page(vec![stage(Some(option("opt-qa", "QA", "pink")))]);
        apply(&mut qa, &released);
        assert_eq!(
            qa.status,
            Some(NotionStatus {
                name: "QA".into(),
                color: "pink".into(),
                group: None
            })
        );
        assert!(!qa.is_settled());

        let mut shipped = page(vec![stage(Some(option(
            "opt-released",
            "Released (v2)",
            "green",
        )))]);
        apply(&mut shipped, &released);
        // By the option's ID: renamed since the rule was set
        assert_eq!(shipped.status.as_ref().unwrap().name, "Released (v2)");
        assert_eq!(
            shipped.status.as_ref().unwrap().group,
            Some(StatusGroup::Complete)
        );
        assert!(shipped.is_settled());

        let mut empty = page(vec![stage(None)]);
        apply(&mut empty, &released);
        assert_eq!(empty.status, None);
    }

    #[test]
    fn a_status_rule_can_count_complete_options_out() {
        let done_only = rule(ColumnKind::Status, "s", &[("opt-deployed", "Deployed")]);
        let status = |id: &str, name: &str, group| {
            let mut page = page(vec![NotionColumn {
                property_id: "s".into(),
                name: "Status".into(),
                value: NotionColumnValue::Status {
                    option: Some(option(id, name, "green")),
                },
            }]);
            page.status = Some(NotionStatus {
                name: name.into(),
                color: "green".into(),
                group: Some(group),
            });
            page
        };
        let mut merged = status("opt-merged", "Merged", StatusGroup::Complete);
        apply(&mut merged, &done_only);
        assert_eq!(merged.status.as_ref().unwrap().group, None);
        let mut doing = status("opt-doing", "Doing", StatusGroup::InProgress);
        apply(&mut doing, &done_only);
        assert_eq!(
            doing.status.as_ref().unwrap().group,
            Some(StatusGroup::InProgress)
        );
        let mut deployed = status("opt-deployed", "Deployed", StatusGroup::Complete);
        apply(&mut deployed, &done_only);
        assert_eq!(
            deployed.status.as_ref().unwrap().group,
            Some(StatusGroup::Complete)
        );
    }

    #[test]
    fn multi_select_and_checkbox_rules() {
        let tags = rule(ColumnKind::MultiSelect, "t", &[("opt-shipped", "shipped")]);
        let tagged = |options: Vec<NotionOptionValue>| {
            page(vec![NotionColumn {
                property_id: "t".into(),
                name: "Tags".into(),
                value: NotionColumnValue::MultiSelect { options },
            }])
        };
        let mut done = tagged(vec![
            option("opt-docs", "docs", "gray"),
            option("opt-shipped", "shipped", "green"),
        ]);
        apply(&mut done, &tags);
        assert_eq!(done.status.as_ref().unwrap().name, "shipped");
        assert_eq!(
            done.status.as_ref().unwrap().group,
            Some(StatusGroup::Complete)
        );
        let mut not_yet = tagged(vec![option("opt-docs", "docs", "gray")]);
        apply(&mut not_yet, &tags);
        assert_eq!(not_yet.status.as_ref().unwrap().name, "docs");
        assert_eq!(not_yet.status.as_ref().unwrap().group, None);

        let shipped = NotionDoneRule {
            property_name: "Shipped".into(),
            ..rule(ColumnKind::Checkbox, "sh", &[])
        };
        let checkbox = |checked| {
            page(vec![NotionColumn {
                property_id: "sh".into(),
                name: "Shipped".into(),
                value: NotionColumnValue::Checkbox { checked },
            }])
        };
        let mut checked = checkbox(true);
        apply(&mut checked, &shipped);
        assert_eq!(checked.status.as_ref().unwrap().name, "Shipped");
        assert_eq!(
            checked.status.as_ref().unwrap().group,
            Some(StatusGroup::Complete)
        );
        let mut unchecked = checkbox(false);
        apply(&mut unchecked, &shipped);
        assert_eq!(unchecked.status, None);
    }

    #[test]
    fn without_the_column_the_page_is_shown_as_notion_tells() {
        let released = rule(ColumnKind::Select, "st", &[("opt-released", "Released")]);
        // Stored before columns were kept, or the column was deleted
        let mut old = page(Vec::new());
        let before = old.status.clone();
        apply(&mut old, &released);
        assert_eq!(old.status, before);
        // Its type changed since
        let mut changed = page(vec![NotionColumn {
            property_id: "st".into(),
            name: "Stage".into(),
            value: NotionColumnValue::Checkbox { checked: true },
        }]);
        apply(&mut changed, &released);
        assert_eq!(changed.status, before);
    }
}
