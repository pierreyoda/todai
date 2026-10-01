CREATE TABLE
  todos (
    -- UUIDv7, generated in Rust.
    id TEXT PRIMARY KEY,
    -- 'YYYY-MM-DD', local date.
    day TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    completed INTEGER NOT NULL DEFAULT 0,
    -- Fractional index key.
    position TEXT NOT NULL,
    completed_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    -- Set on delete; row is kept.
    deleted_at INTEGER
  );

CREATE INDEX idx_todos_day ON todos (day, position)
WHERE
  deleted_at IS NULL;