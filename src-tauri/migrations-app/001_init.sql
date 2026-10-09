CREATE TABLE
  workspaces (
    -- UUIDv7.
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    -- Absolute and canonical path of the workspace database.
    path TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    last_opened_at INTEGER
  );

-- Single row (id = 1).
CREATE TABLE
  app_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    active_workspace_id TEXT REFERENCES workspaces (id) ON DELETE SET NULL
  );

INSERT INTO
  app_state (id)
VALUES
  (1);
