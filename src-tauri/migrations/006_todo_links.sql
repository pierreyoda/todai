CREATE TABLE
  todo_links (
    -- UUIDv7, generated in Rust.
    id TEXT PRIMARY KEY,
    -- Kept while its todo is deleted, so that it comes back when restored.
    todo_id TEXT NOT NULL REFERENCES todos (id) ON DELETE CASCADE,
    -- 'github_pull', 'github_issue' or 'url': recognized from the URL in Rust.
    kind TEXT NOT NULL CHECK (kind IN ('github_pull', 'github_issue', 'url')),
    -- Normalized in Rust, e.g. a pull request's URL without its tab (`/files`...).
    url TEXT NOT NULL,
    -- The pull request or issue: set exactly for the GitHub kinds.
    github_owner TEXT,
    github_repo TEXT,
    github_number INTEGER CHECK (github_number > 0),
    created_at INTEGER NOT NULL,
    CHECK ((kind = 'url') = (github_owner IS NULL)),
    CHECK ((github_owner IS NULL) = (github_repo IS NULL)),
    CHECK ((github_owner IS NULL) = (github_number IS NULL)),
    -- Also covers lookups (and cascades) by todo.
    UNIQUE (todo_id, url)
  );
