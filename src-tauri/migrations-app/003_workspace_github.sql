-- The GitHub CLI (`gh`) account the workspace uses on github.com, by login; NULL while not connected. No token is
-- stored: it's asked to `gh` for this account when needed.
ALTER TABLE workspaces
ADD COLUMN github_login TEXT CHECK (github_login <> '');
