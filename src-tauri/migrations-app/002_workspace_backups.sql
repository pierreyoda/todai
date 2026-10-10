-- Whether a backup is made when the workspace is opened, if its last automatic one is from another (local) day.
ALTER TABLE workspaces
ADD COLUMN auto_backup INTEGER NOT NULL DEFAULT 1 CHECK (auto_backup IN (0, 1));

-- Automatic backups kept: older ones are deleted, with their file. Other backups are only deleted manually.
ALTER TABLE workspaces
ADD COLUMN auto_backup_keep INTEGER NOT NULL DEFAULT 7 CHECK (auto_backup_keep BETWEEN 1 AND 100);

CREATE TABLE
  workspace_backups (
    -- UUIDv7.
    id TEXT PRIMARY KEY,
    -- Rows go with their workspace; their files are only deleted if asked when removing it.
    workspace_id TEXT NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    -- Absolute and canonical path of the backup file: <app data>/backups/<workspace id>/<file>.sqlite3.
    path TEXT NOT NULL UNIQUE,
    -- 'manual', 'automatic' (pruned beyond the workspace's auto_backup_keep), 'pre_migration' (before migrating the
    -- workspace's database to a newer version of todai) or 'pre_restore' (its state before restoring another backup).
    kind TEXT NOT NULL CHECK (kind IN ('manual', 'automatic', 'pre_migration', 'pre_restore')),
    created_at INTEGER NOT NULL
  );

-- A workspace's backups by date; also covers the cascade from workspaces.
CREATE INDEX idx_workspace_backups_workspace ON workspace_backups (workspace_id, created_at);
