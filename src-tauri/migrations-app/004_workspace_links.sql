-- How the workspace's links behave, each on by default:
-- whether their GitHub statuses are refreshed on their own while todai is open, rather than only when asked;
ALTER TABLE workspaces
ADD COLUMN links_auto_refresh INTEGER NOT NULL DEFAULT 1 CHECK (links_auto_refresh IN (0, 1));

-- whether a todo whose pull request is merged (or issue completed) offers to be marked as done;
ALTER TABLE workspaces
ADD COLUMN links_suggest_done INTEGER NOT NULL DEFAULT 1 CHECK (links_suggest_done IN (0, 1));

-- whether pasting a pull request's or issue's link as a new todo's title offers to create it from the link.
ALTER TABLE workspaces
ADD COLUMN links_paste_to_create INTEGER NOT NULL DEFAULT 1 CHECK (links_paste_to_create IN (0, 1));
