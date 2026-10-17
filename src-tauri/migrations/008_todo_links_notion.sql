-- Links to Notion pages (or whole databases): kind 'notion', with the page's ID. Rebuilt, as SQLite can't change a
-- table's CHECKs: the same rows, with the new column.
CREATE TABLE
  todo_links_new (
    id TEXT PRIMARY KEY,
    todo_id TEXT NOT NULL REFERENCES todos (id) ON DELETE CASCADE,
    -- 'github_pull', 'github_issue', 'notion' or 'url': recognized from the URL in Rust.
    kind TEXT NOT NULL CHECK (kind IN ('github_pull', 'github_issue', 'notion', 'url')),
    url TEXT NOT NULL,
    -- The pull request or issue: set exactly for the GitHub kinds.
    github_owner TEXT,
    github_repo TEXT,
    github_number INTEGER CHECK (github_number > 0),
    -- The Notion page's (or database's) ID, 32 lowercase hex digits: set exactly for the Notion kind.
    notion_id TEXT CHECK (length(notion_id) = 32),
    created_at INTEGER NOT NULL,
    snapshot TEXT,
    fetched_at INTEGER,
    fetch_error TEXT CHECK (fetch_error IN ('no_access', 'sso_required')),
    CHECK ((kind IN ('github_pull', 'github_issue')) = (github_owner IS NOT NULL)),
    CHECK ((github_owner IS NULL) = (github_repo IS NULL)),
    CHECK ((github_owner IS NULL) = (github_number IS NULL)),
    CHECK ((kind = 'notion') = (notion_id IS NOT NULL)),
    UNIQUE (todo_id, url)
  );

INSERT INTO
  todo_links_new (id, todo_id, kind, url, github_owner, github_repo, github_number, created_at, snapshot, fetched_at, fetch_error)
SELECT
  id, todo_id, kind, url, github_owner, github_repo, github_number, created_at, snapshot, fetched_at, fetch_error
FROM
  todo_links;

DROP TABLE todo_links;

ALTER TABLE todo_links_new RENAME TO todo_links;

-- The same page pasted twice, under different URLs (e.g. with and without its slug), is one link.
CREATE UNIQUE INDEX idx_todo_links_notion ON todo_links (todo_id, notion_id) WHERE notion_id IS NOT NULL;
