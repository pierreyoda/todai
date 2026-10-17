//! What todai keeps of a Notion page or database (see `links::snapshot`), and how it's read from Notion's answers.

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::links::{self, notion::NotionRef, LinkKind, ParsedLink};

/// How many assignees and tags are kept.
const MAX_PEOPLE: usize = 5;
const MAX_TAGS: usize = 10;

/// A page, as Notion last told about it: its title, and the properties worth showing if it's in a database.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NotionPageSnapshot {
    /// Empty for an untitled page.
    pub title: String,
    /// Its emoji icon, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Its database's unique ID, e.g. TOD-31, if it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_id: Option<UniqueId>,
    /// Its Status, if its database has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<NotionStatus>,
    /// The first few people of its Assignee (or first people) property.
    pub assignees: Vec<String>,
    /// The start of its Due (or first date) property: `YYYY-MM-DD`, or an RFC 3339 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    /// The first few options of its Tags (or first multi-select) property.
    pub tags: Vec<NotionOption>,
    pub last_edited_at: Timestamp,
    /// Whether it's in Notion's trash.
    pub in_trash: bool,
    /// The database it's in, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<NotionParentDatabase>,
    /// The value of each of its columns that can tell it's done (see `rules`): absent from snapshots stored before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<NotionColumn>,
}

/// The value of a page's column that can tell it's done: a status, a select or multi-select, or a checkbox.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotionColumn {
    /// Notion's ID of the property, which stays the same when renamed.
    pub property_id: String,
    pub name: String,
    pub value: NotionColumnValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum NotionColumnValue {
    Status { option: Option<NotionOptionValue> },
    Select { option: Option<NotionOptionValue> },
    MultiSelect { options: Vec<NotionOptionValue> },
    Checkbox { checked: bool },
}

/// An option picked in a status, select or multi-select column.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NotionOptionValue {
    pub id: String,
    pub name: String,
    pub color: String,
}

/// A column of a database that can tell its pages are done, with every option it offers.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaColumn {
    pub property_id: String,
    pub name: String,
    pub kind: ColumnKind,
    /// None for a checkbox.
    pub options: Vec<NotionOptionValue>,
}

/// What kind of column a done rule reads. Serialized like the `property_kind` column of `notion_done_rules`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ColumnKind {
    Status,
    Select,
    MultiSelect,
    Checkbox,
}

impl ColumnKind {
    pub fn to_db(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Select => "select",
            Self::MultiSelect => "multi_select",
            Self::Checkbox => "checkbox",
        }
    }

    pub fn from_db(kind: &str) -> Option<Self> {
        match kind {
            "status" => Some(Self::Status),
            "select" => Some(Self::Select),
            "multi_select" => Some(Self::MultiSelect),
            "checkbox" => Some(Self::Checkbox),
            _ => None,
        }
    }
}

impl NotionColumnValue {
    pub fn kind(&self) -> ColumnKind {
        match self {
            Self::Status { .. } => ColumnKind::Status,
            Self::Select { .. } => ColumnKind::Select,
            Self::MultiSelect { .. } => ColumnKind::MultiSelect,
            Self::Checkbox { .. } => ColumnKind::Checkbox,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UniqueId {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    pub number: u64,
}

impl UniqueId {
    /// E.g. `TOD-31`, or `31` without a prefix.
    pub fn label(&self) -> String {
        match &self.prefix {
            Some(prefix) => format!("{prefix}-{}", self.number),
            None => self.number.to_string(),
        }
    }
}

/// A Status property's value: its option, and the group Notion files it under.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotionStatus {
    pub name: String,
    /// One of Notion's: `default`, `gray`, `brown`, `orange`, `yellow`, `green`, `blue`, `purple`, `pink` or `red`.
    pub color: String,
    /// `None` when its database's schema couldn't be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<StatusGroup>,
}

/// The three groups a Status property's options are filed under.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusGroup {
    ToDo,
    InProgress,
    Complete,
}

/// A select option, e.g. a tag.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NotionOption {
    pub name: String,
    pub color: String,
}

/// The database a page is in: its data source, in Notion's terms.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotionParentDatabase {
    pub data_source_id: String,
    /// `None` when its schema couldn't be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// A whole database, as Notion last told about it: a link to one shows its title, without a status.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NotionDatabaseSnapshot {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub last_edited_at: Timestamp,
    pub in_trash: bool,
}

impl NotionPageSnapshot {
    /// Whether it's done with: complete, or in the trash.
    pub fn is_settled(&self) -> bool {
        self.in_trash
            || self
                .status
                .as_ref()
                .is_some_and(|status| status.group == Some(StatusGroup::Complete))
    }

    /// How it's labelled on a link: its unique ID, else its title.
    pub fn label(&self) -> Option<String> {
        self.unique_id
            .as_ref()
            .map(UniqueId::label)
            .or_else(|| (!self.title.is_empty()).then(|| self.title.clone()))
    }
}

/// What's known of a database's schema (a data source's, in Notion's terms): what its pages' status groups mean, and
/// the columns that can tell they're done.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataSourceInfo {
    pub title: String,
    /// Its Status property's groups, each with the IDs of its options.
    pub status_groups: Vec<(StatusGroup, Vec<String>)>,
    /// By name.
    pub columns: Vec<SchemaColumn>,
}

impl DataSourceInfo {
    /// The group of the status option `option_id`, if the schema has it.
    pub fn group_of(&self, option_id: &str) -> Option<StatusGroup> {
        self.status_groups
            .iter()
            .find(|(_, options)| options.iter().any(|option| option == option_id))
            .map(|(group, _)| *group)
    }
}

// What Notion answers: only what's used, the rest left as `Value` or skipped.

#[derive(Deserialize)]
pub(super) struct RawPage {
    url: Option<String>,
    #[serde(default)]
    in_trash: bool,
    last_edited_time: Timestamp,
    icon: Option<RawIcon>,
    parent: Option<RawParent>,
    #[serde(default)]
    properties: BTreeMap<String, RawProperty>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawIcon {
    Emoji {
        emoji: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawParent {
    DataSourceId {
        data_source_id: String,
    },
    #[serde(other)]
    Other,
}

/// A page's property, by type: the value is under a key named after the type.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawProperty {
    Title {
        title: Vec<RawRichText>,
    },
    Status {
        #[serde(default)]
        id: String,
        status: Option<NotionOptionValue>,
    },
    Select {
        #[serde(default)]
        id: String,
        select: Option<NotionOptionValue>,
    },
    Checkbox {
        #[serde(default)]
        id: String,
        checkbox: bool,
    },
    People {
        people: Vec<RawPerson>,
    },
    Date {
        date: Option<RawDate>,
    },
    MultiSelect {
        #[serde(default)]
        id: String,
        multi_select: Vec<NotionOptionValue>,
    },
    UniqueId {
        unique_id: UniqueId,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct RawRichText {
    plain_text: String,
}

#[derive(Deserialize)]
struct RawPerson {
    name: Option<String>,
}

#[derive(Deserialize)]
struct RawDate {
    start: String,
}

fn plain_text(rich_text: &[RawRichText]) -> String {
    rich_text
        .iter()
        .map(|text| text.plain_text.as_str())
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The property named like one of `preferred` (whatever its case), else the first one `pick` accepts.
fn property<'a, P, T>(
    properties: &'a BTreeMap<String, P>,
    preferred: &[&str],
    pick: impl Fn(&'a P) -> Option<T>,
) -> Option<T> {
    let preferred = preferred.iter().find_map(|name| {
        properties
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .and_then(|(_, property)| pick(property))
    });
    preferred.or_else(|| properties.values().find_map(pick))
}

/// What `page` tells of a page: before its status is filed under its group, which takes its database's schema.
pub(super) struct ParsedPage {
    pub link: ParsedLink,
    pub snapshot: NotionPageSnapshot,
    /// The ID of its Status option, to file under its group.
    pub status_option_id: Option<String>,
}

/// Reads the page `id`, as Notion answered it.
pub(super) fn parse_page(id: &NotionRef, page: Value) -> serde_json::Result<ParsedPage> {
    let page: RawPage = serde_json::from_value(page)?;
    let properties = &page.properties;
    let title = property(properties, &["title", "name"], |property| match property {
        RawProperty::Title { title } => Some(plain_text(title)),
        _ => None,
    })
    .unwrap_or_default();
    let status = property(
        properties,
        &["status", "state"],
        |property| match property {
            RawProperty::Status { status, .. } => status.as_ref(),
            _ => None,
        },
    );
    let assignees = property(
        properties,
        &["assignee", "assignees", "owner", "owners", "assigned to"],
        |property| match property {
            RawProperty::People { people } => Some(people),
            _ => None,
        },
    )
    .map(|people| {
        people
            .iter()
            .filter_map(|person| person.name.clone())
            .take(MAX_PEOPLE)
            .collect()
    })
    .unwrap_or_default();
    let due = property(
        properties,
        &["due", "due date", "deadline", "date"],
        |property| match property {
            RawProperty::Date { date } => date.as_ref().map(|date| date.start.clone()),
            _ => None,
        },
    );
    let tags = property(properties, &["tags", "labels"], |property| match property {
        RawProperty::MultiSelect { multi_select, .. } => Some(multi_select),
        _ => None,
    })
    .map(|options| {
        options
            .iter()
            .take(MAX_TAGS)
            .map(|option| NotionOption {
                name: option.name.clone(),
                color: option.color.clone(),
            })
            .collect()
    })
    .unwrap_or_default();
    let unique_id = property(properties, &["id"], |property| match property {
        RawProperty::UniqueId { unique_id } => Some(unique_id.clone()),
        _ => None,
    });
    let database = match page.parent {
        Some(RawParent::DataSourceId { data_source_id }) => Some(NotionParentDatabase {
            data_source_id,
            title: None,
        }),
        _ => None,
    };
    let columns = properties
        .iter()
        .filter_map(|(name, property)| {
            let (property_id, value) = match property {
                RawProperty::Status { id, status } => (
                    id,
                    NotionColumnValue::Status {
                        option: status.clone(),
                    },
                ),
                RawProperty::Select { id, select } => (
                    id,
                    NotionColumnValue::Select {
                        option: select.clone(),
                    },
                ),
                RawProperty::MultiSelect { id, multi_select } => (
                    id,
                    NotionColumnValue::MultiSelect {
                        options: multi_select.clone(),
                    },
                ),
                RawProperty::Checkbox { id, checkbox } => {
                    (id, NotionColumnValue::Checkbox { checked: *checkbox })
                }
                _ => return None,
            };
            Some(NotionColumn {
                property_id: property_id.clone(),
                name: name.clone(),
                value,
            })
        })
        .collect();
    let snapshot = NotionPageSnapshot {
        title,
        icon: match page.icon {
            Some(RawIcon::Emoji { emoji }) => Some(emoji),
            _ => None,
        },
        unique_id,
        status: status.map(|status| NotionStatus {
            name: status.name.clone(),
            color: status.color.clone(),
            group: None,
        }),
        assignees,
        due,
        tags,
        last_edited_at: page.last_edited_time,
        in_trash: page.in_trash,
        database,
        columns,
    };
    Ok(ParsedPage {
        link: link_at(id, page.url.as_deref()),
        snapshot,
        status_option_id: status.map(|status| status.id.clone()),
    })
}

/// The link to the item `id`, where Notion says it is (`url`), else as normalized from its ID.
fn link_at(id: &NotionRef, url: Option<&str>) -> ParsedLink {
    url.and_then(|url| links::parse(url).ok())
        .filter(|link| link.kind == LinkKind::Notion && link.notion.as_ref() == Some(id))
        .unwrap_or_else(|| ParsedLink {
            kind: LinkKind::Notion,
            url: links::notion::url(&id.id, None),
            label: links::notion::UNTITLED.into(),
            github: None,
            notion: Some(id.clone()),
        })
}

#[derive(Deserialize)]
struct RawDataSource {
    #[serde(default)]
    title: Vec<RawRichText>,
    #[serde(default)]
    properties: BTreeMap<String, RawSchemaProperty>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawSchemaProperty {
    Status {
        #[serde(default)]
        id: String,
        status: RawStatusSchema,
    },
    Select {
        #[serde(default)]
        id: String,
        select: RawOptionsSchema,
    },
    MultiSelect {
        #[serde(default)]
        id: String,
        multi_select: RawOptionsSchema,
    },
    Checkbox {
        #[serde(default)]
        id: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct RawOptionsSchema {
    #[serde(default)]
    options: Vec<NotionOptionValue>,
}

#[derive(Deserialize)]
struct RawStatusSchema {
    #[serde(default)]
    options: Vec<NotionOptionValue>,
    #[serde(default)]
    groups: Vec<RawStatusGroup>,
}

#[derive(Deserialize)]
struct RawStatusGroup {
    name: String,
    #[serde(default)]
    option_ids: Vec<String>,
}

/// Reads a database's schema (a data source's), as Notion answered it.
pub(super) fn parse_data_source(value: Value) -> serde_json::Result<DataSourceInfo> {
    let source: RawDataSource = serde_json::from_value(value)?;
    let groups = property(
        &source.properties,
        &["status", "state"],
        |property| match property {
            RawSchemaProperty::Status { status, .. } => Some(&status.groups),
            _ => None,
        },
    )
    .map(|groups| {
        groups
            .iter()
            .enumerate()
            .filter_map(|(i, group)| {
                Some((status_group(&group.name, i)?, group.option_ids.clone()))
            })
            .collect()
    })
    .unwrap_or_default();
    let columns = source
        .properties
        .iter()
        .filter_map(|(name, property)| {
            let (property_id, kind, options) = match property {
                RawSchemaProperty::Status { id, status } => {
                    (id, ColumnKind::Status, status.options.clone())
                }
                RawSchemaProperty::Select { id, select } => {
                    (id, ColumnKind::Select, select.options.clone())
                }
                RawSchemaProperty::MultiSelect { id, multi_select } => {
                    (id, ColumnKind::MultiSelect, multi_select.options.clone())
                }
                RawSchemaProperty::Checkbox { id } => (id, ColumnKind::Checkbox, Vec::new()),
                RawSchemaProperty::Other => return None,
            };
            Some(SchemaColumn {
                property_id: property_id.clone(),
                name: name.clone(),
                kind,
                options,
            })
        })
        .collect();
    Ok(DataSourceInfo {
        title: plain_text(&source.title),
        status_groups: groups,
        columns,
    })
}

/// Which group a Status property's group named `name` is, at `position` among them: by its name as Notion creates them
/// (which the API doesn't translate), else by its position (To-do, In progress, Complete).
fn status_group(name: &str, position: usize) -> Option<StatusGroup> {
    match name.to_ascii_lowercase().as_str() {
        "to-do" | "to do" | "todo" => Some(StatusGroup::ToDo),
        "in progress" => Some(StatusGroup::InProgress),
        "complete" | "completed" | "done" => Some(StatusGroup::Complete),
        _ => match position {
            0 => Some(StatusGroup::ToDo),
            1 => Some(StatusGroup::InProgress),
            2 => Some(StatusGroup::Complete),
            _ => None,
        },
    }
}

#[derive(Deserialize)]
struct RawDatabase {
    url: Option<String>,
    #[serde(default)]
    title: Vec<RawRichText>,
    icon: Option<RawIcon>,
    #[serde(default)]
    in_trash: bool,
    last_edited_time: Timestamp,
}

/// Reads the whole database `id`, as Notion answered it.
pub(super) fn parse_database(
    id: &NotionRef,
    value: Value,
) -> serde_json::Result<(ParsedLink, NotionDatabaseSnapshot)> {
    let database: RawDatabase = serde_json::from_value(value)?;
    let snapshot = NotionDatabaseSnapshot {
        title: plain_text(&database.title),
        icon: match database.icon {
            Some(RawIcon::Emoji { emoji }) => Some(emoji),
            _ => None,
        },
        last_edited_at: database.last_edited_time,
        in_trash: database.in_trash,
    };
    Ok((link_at(id, database.url.as_deref()), snapshot))
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::json;

    use super::*;

    pub(crate) const ID: &str = "8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5";

    fn id() -> NotionRef {
        NotionRef { id: ID.into() }
    }

    /// A ticket, in a Tickets database.
    pub(crate) fn ticket_json() -> Value {
        json!({
            "object": "page", "id": "8c1d2e3f-4a5b-46c7-d8e9-f0a1b2c3d4e5",
            "created_time": "2026-10-01T09:00:00.000Z", "last_edited_time": "2026-10-15T10:12:00.000Z",
            "in_trash": false, "icon": { "type": "emoji", "emoji": "🎫" },
            "parent": { "type": "data_source_id", "data_source_id": "ds1" },
            "url": "https://www.notion.so/Add-links-metadata-to-todos-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
            "properties": {
                "Name": { "id": "title", "type": "title", "title": [
                    { "type": "text", "plain_text": "Add links metadata " }, { "type": "text", "plain_text": "to todos" }
                ]},
                "Status": { "id": "s", "type": "status", "status": { "id": "opt-doing", "name": "In progress", "color": "blue" } },
                "Assignee": { "id": "a", "type": "people", "people": [
                    { "object": "user", "id": "u1", "name": "Pierre-Yves D." }, { "object": "user", "id": "u2", "name": "Camille R." }
                ]},
                "Due": { "id": "d", "type": "date", "date": { "start": "2026-10-16", "end": null } },
                "Tags": { "id": "t", "type": "multi_select", "multi_select": [
                    { "id": "x", "name": "links", "color": "orange" }, { "id": "y", "name": "backend", "color": "purple" }
                ]},
                "ID": { "id": "i", "type": "unique_id", "unique_id": { "prefix": "TOD", "number": 31 } },
                "Priority": { "id": "p", "type": "select", "select": { "id": "opt-high", "name": "High", "color": "red" } },
                "Stage": { "id": "st", "type": "select", "select": { "id": "opt-qa", "name": "QA", "color": "pink" } },
                "Shipped": { "id": "sh", "type": "checkbox", "checkbox": false },
                "Rollup": { "id": "r", "type": "rollup", "rollup": { "type": "number", "number": 3 } }
            }
        })
    }

    /// The ticket, as read, its Status filed under In progress.
    pub(crate) fn parsed_ticket() -> NotionPageSnapshot {
        let mut snapshot = parse_page(&id(), ticket_json()).unwrap().snapshot;
        if let Some(status) = &mut snapshot.status {
            status.group = Some(StatusGroup::InProgress);
        }
        snapshot
    }

    pub(crate) fn tickets_schema_json() -> Value {
        json!({
            "object": "data_source", "id": "ds1",
            "title": [{ "type": "text", "plain_text": "Tickets" }],
            "properties": {
                "Status": { "id": "s", "type": "status", "status": {
                    "options": [
                        { "id": "opt-todo", "name": "Not started", "color": "default" },
                        { "id": "opt-doing", "name": "In progress", "color": "blue" },
                        { "id": "opt-done", "name": "Done", "color": "green" }
                    ],
                    "groups": [
                        { "id": "g1", "name": "To-do", "color": "gray", "option_ids": ["opt-todo"] },
                        { "id": "g2", "name": "In progress", "color": "blue", "option_ids": ["opt-doing"] },
                        { "id": "g3", "name": "Complete", "color": "green", "option_ids": ["opt-done"] }
                    ]
                }},
                "Name": { "id": "title", "type": "title", "title": {} },
                "Stage": { "id": "st", "type": "select", "select": { "options": [
                    { "id": "opt-dev", "name": "Dev", "color": "blue" },
                    { "id": "opt-qa", "name": "QA", "color": "pink" },
                    { "id": "opt-released", "name": "Released", "color": "green" }
                ]}},
                "Shipped": { "id": "sh", "type": "checkbox", "checkbox": {} },
                "Due": { "id": "d", "type": "date", "date": {} }
            }
        })
    }

    #[test]
    fn reads_a_page_in_a_database() {
        let parsed = parse_page(&id(), ticket_json()).unwrap();
        assert_eq!(
            parsed.link.url,
            "https://www.notion.so/Add-links-metadata-to-todos-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5"
        );
        assert_eq!(parsed.status_option_id.as_deref(), Some("opt-doing"));
        let snapshot = parsed.snapshot;
        assert_eq!(snapshot.title, "Add links metadata to todos");
        assert_eq!(snapshot.icon.as_deref(), Some("🎫"));
        assert_eq!(snapshot.label().as_deref(), Some("TOD-31"));
        assert_eq!(
            snapshot.status,
            Some(NotionStatus {
                name: "In progress".into(),
                color: "blue".into(),
                group: None
            })
        );
        assert_eq!(snapshot.assignees, ["Pierre-Yves D.", "Camille R."]);
        assert_eq!(snapshot.due.as_deref(), Some("2026-10-16"));
        assert_eq!(snapshot.tags.len(), 2);
        assert_eq!(
            snapshot
                .database
                .as_ref()
                .map(|db| db.data_source_id.as_str()),
            Some("ds1")
        );
        assert!(!snapshot.in_trash);
        assert!(!snapshot.is_settled());
    }

    #[test]
    fn keeps_the_columns_that_can_tell_a_page_is_done() {
        let snapshot = parse_page(&id(), ticket_json()).unwrap().snapshot;
        let column = |name: &str| {
            snapshot
                .columns
                .iter()
                .find(|column| column.name == name)
                .map(|column| (column.property_id.as_str(), column.value.clone()))
        };
        assert_eq!(
            column("Stage"),
            Some((
                "st",
                NotionColumnValue::Select {
                    option: Some(NotionOptionValue {
                        id: "opt-qa".into(),
                        name: "QA".into(),
                        color: "pink".into()
                    })
                }
            ))
        );
        assert_eq!(
            column("Shipped"),
            Some(("sh", NotionColumnValue::Checkbox { checked: false }))
        );
        assert!(matches!(
            column("Status"),
            Some(("s", NotionColumnValue::Status { option: Some(_) }))
        ));
        assert!(matches!(
            column("Tags"),
            Some(("t", NotionColumnValue::MultiSelect { options })) if options.len() == 2
        ));
        assert_eq!(column("Due"), None);
        // Stored then read again
        let stored = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(
            serde_json::from_value::<NotionPageSnapshot>(stored).unwrap(),
            snapshot
        );
    }

    #[test]
    fn a_schema_lists_the_columns_that_can_tell_its_pages_are_done() {
        let info = parse_data_source(tickets_schema_json()).unwrap();
        let names: Vec<(&str, ColumnKind, usize)> = info
            .columns
            .iter()
            .map(|column| (column.name.as_str(), column.kind, column.options.len()))
            .collect();
        assert_eq!(
            names,
            [
                ("Shipped", ColumnKind::Checkbox, 0),
                ("Stage", ColumnKind::Select, 3),
                ("Status", ColumnKind::Status, 3)
            ]
        );
    }

    #[test]
    fn reads_a_plain_page_and_a_trashed_one() {
        let plain = json!({
            "object": "page", "id": ID, "last_edited_time": "2026-10-14T08:00:00.000Z", "in_trash": true,
            "parent": { "type": "workspace", "workspace": true }, "url": "https://www.notion.so/0-3-roadmap-8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
            "properties": { "title": { "id": "title", "type": "title", "title": [{ "plain_text": "0.3 roadmap" }] } }
        });
        let parsed = parse_page(&id(), plain).unwrap();
        assert_eq!(parsed.snapshot.title, "0.3 roadmap");
        assert_eq!(parsed.snapshot.label().as_deref(), Some("0.3 roadmap"));
        assert_eq!(parsed.snapshot.status, None);
        assert_eq!(parsed.snapshot.database, None);
        assert!(parsed.snapshot.in_trash);
        assert!(parsed.snapshot.is_settled());
        // Where Notion says it is, if it's the same page
        assert_eq!(parsed.link.label, "0 3 roadmap");
        let elsewhere = parse_page(
            &id(),
            json!({ "object": "page", "id": ID, "last_edited_time": "2026-10-14T08:00:00.000Z",
                "url": "https://www.notion.so/Other-00000000000000000000000000000000", "properties": {} }),
        )
        .unwrap();
        assert_eq!(elsewhere.link.url, format!("https://www.notion.so/{ID}"));
        assert_eq!(elsewhere.snapshot.label(), None);
    }

    #[test]
    fn reads_a_schema_s_status_groups() {
        let info = parse_data_source(tickets_schema_json()).unwrap();
        assert_eq!(info.title, "Tickets");
        assert_eq!(info.group_of("opt-doing"), Some(StatusGroup::InProgress));
        assert_eq!(info.group_of("opt-done"), Some(StatusGroup::Complete));
        assert_eq!(info.group_of("unknown"), None);
        // Renamed groups: by their position
        assert_eq!(status_group("À faire", 0), Some(StatusGroup::ToDo));
        assert_eq!(status_group("Terminé", 2), Some(StatusGroup::Complete));
        assert_eq!(status_group("Done", 7), Some(StatusGroup::Complete));
        assert_eq!(status_group("Later", 3), None);
    }

    #[test]
    fn reads_a_whole_database() {
        let (link, snapshot) = parse_database(
            &id(),
            json!({ "object": "database", "id": ID, "title": [{ "plain_text": "Tickets" }], "in_trash": false,
                "last_edited_time": "2026-10-14T08:00:00.000Z", "url": "https://www.notion.so/8c1d2e3f4a5b46c7d8e9f0a1b2c3d4e5",
                "data_sources": [{ "id": "ds1", "name": "Tickets" }] }),
        )
        .unwrap();
        assert_eq!(snapshot.title, "Tickets");
        assert_eq!(link.url, format!("https://www.notion.so/{ID}"));
    }
}
