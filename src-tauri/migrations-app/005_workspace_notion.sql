-- The Notion user whose personal access token the workspace uses, as Notion told when it was connected: NULL while not
-- connected to Notion. The token itself is kept in the macOS Keychain (service `todai.notion`, account: the workspace's
-- ID), never here: a backup restored as a new workspace starts unconnected.
ALTER TABLE workspaces
ADD COLUMN notion_user_id TEXT CHECK (notion_user_id <> '');

-- Their name and email, if Notion tells.
ALTER TABLE workspaces
ADD COLUMN notion_user_name TEXT;

ALTER TABLE workspaces
ADD COLUMN notion_user_email TEXT;

-- The Notion workspace the token is for, if Notion tells.
ALTER TABLE workspaces
ADD COLUMN notion_workspace_name TEXT;
