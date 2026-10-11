-- What GitHub last told about a GitHub link's pull request or issue, as JSON (see `github::snapshot`). Kept when
-- fetching it again fails, so that the link still shows its last known status.
ALTER TABLE todo_links ADD COLUMN snapshot TEXT;

-- When GitHub last answered about it, whether it could see it or not.
ALTER TABLE todo_links ADD COLUMN fetched_at INTEGER;

-- Why GitHub's last answer had nothing about it: 'no_access' (missing, deleted, or private to others) or
-- 'sso_required' (its organization requires single sign-on, which the token isn't authorized for).
ALTER TABLE todo_links ADD COLUMN fetch_error TEXT CHECK (fetch_error IN ('no_access', 'sso_required'));
