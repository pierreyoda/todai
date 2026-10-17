-- Which values of which column of a Notion database mean its pages are done: one rule per database (per data source,
-- in Notion's terms), overriding Notion's own Status groups (a page is done once in its Complete group, by default).
CREATE TABLE
  notion_done_rules (
    data_source_id TEXT PRIMARY KEY,
    -- The database's title when the rule was set, as shown in the settings.
    database_title TEXT NOT NULL,
    -- The column, by Notion's ID (which stays the same when renamed), with its name when the rule was set.
    property_id TEXT NOT NULL CHECK (property_id <> ''),
    property_name TEXT NOT NULL,
    property_kind TEXT NOT NULL CHECK (property_kind IN ('status', 'select', 'multi_select', 'checkbox')),
    -- The options meaning done, as a JSON array of {"id", "name"}: by their ID, which stays the same when renamed. Empty
    -- for a checkbox, which is done when checked.
    done_options TEXT NOT NULL,
    updated_at INTEGER NOT NULL
  );
