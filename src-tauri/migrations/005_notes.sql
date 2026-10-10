CREATE TABLE
  notes (
    -- 'YYYY-MM-DD', local date: at most one note per day.
    day TEXT PRIMARY KEY,
    -- Markdown. A note emptied (blank) is deleted rather than kept empty.
    content TEXT NOT NULL CHECK (content <> ''),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
  );
