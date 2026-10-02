CREATE TABLE
  tags (
    -- UUIDv7, generated in Rust.
    id TEXT PRIMARY KEY,
    -- Trimmed in Rust; see idx_tags_name for uniqueness.
    name TEXT NOT NULL COLLATE NOCASE,
    -- 0xRRGGBB.
    color INTEGER NOT NULL CHECK (color BETWEEN 0 AND 0xFFFFFF),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    -- Set on delete; row and its todo_tags are kept.
    deleted_at INTEGER
  );

-- Unique regardless of (ASCII) case among non-deleted tags, so a deleted tag's name can be reused.
CREATE UNIQUE INDEX idx_tags_name ON tags (name)
WHERE
  deleted_at IS NULL;

CREATE TABLE
  todo_tags (
    todo_id TEXT NOT NULL REFERENCES todos (id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (todo_id, tag_id)
  ) WITHOUT ROWID;

-- The primary key covers lookups by todo; this one covers lookups (and cascades) by tag.
CREATE INDEX idx_todo_tags_tag ON todo_tags (tag_id);