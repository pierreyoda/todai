import type { UUID } from "node:crypto";

import type { IntegrationProblem } from "#core/integrations";

/** Whose Notion personal access token a workspace uses, as Notion told when it was connected. */
export interface NotionAccount {
  /** Notion's ID of the user. */
  userId: string;
  name?: string;
  email?: string;
  /** The Notion workspace the token is for, if Notion tells. */
  workspaceName?: string;
}

/** A workspace, with its own database. */
export interface Workspace {
  /** UUID v7. */
  id: UUID;
  name: string;
  /** Absolute path of its database. */
  path: string;
  isActive: boolean;
  /** Whether its database can be used: open for the active workspace, existing for the others. */
  available: boolean;
  createdAt: Date;
  lastOpenedAt?: Date;
  /** When its last backup was made, whatever its kind. */
  lastBackupAt?: Date;
  /** Whether it's backed up automatically, once a day: when it's opened, then hourly checks while it's open. */
  autoBackup: boolean;
  /** Automatic backups kept, from 1 to 100: older ones are deleted. */
  autoBackupKeep: number;
  /** The GitHub CLI account it uses on github.com, if connected to GitHub. */
  githubLogin?: string;
  /**
   * Why its GitHub connection needs the user, if it did the last time GitHub was asked (since todai was opened): the
   * GitHub CLI is missing, or signed out of its account.
   */
  githubAttention?: Extract<
    IntegrationProblem,
    { kind: "gh_missing" | "signed_out" }
  >;
  /** Whose Notion personal access token it uses, if connected to Notion: the token itself is in the macOS Keychain. */
  notion?: NotionAccount;
  /**
   * Why its Notion connection needs the user, if it did the last time Notion was asked (since todai was opened): its
   * token was refused, or isn't in the Keychain anymore.
   */
  notionAttention?: Extract<
    IntegrationProblem,
    { kind: "token_refused" | "token_missing" }
  >;
  /** Whether its links' statuses are refreshed on their own while todai is open, rather than only when asked. */
  linksAutoRefresh: boolean;
  /** Whether a todo whose pull request is merged (or issue completed) offers to be marked as done. */
  linksSuggestDone: boolean;
  /** Whether pasting a pull request's or issue's link as a new todo's title offers to create it from the link. */
  linksPasteToCreate: boolean;
}
