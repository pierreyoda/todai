import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";
import type { GitHubProblem } from "./types";

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
  githubAttention?: Extract<GitHubProblem, { kind: "gh_missing" | "signed_out" }>;
  /** Whether its links' GitHub statuses are refreshed on their own while todai is open, rather than only when asked. */
  linksAutoRefresh: boolean;
  /** Whether a todo whose pull request is merged (or issue completed) offers to be marked as done. */
  linksSuggestDone: boolean;
  /** Whether pasting a pull request's or issue's link as a new todo's title offers to create it from the link. */
  linksPasteToCreate: boolean;
}

/** An account the GitHub CLI (`gh`) knows on github.com. */
export interface GhAccount {
  login: string;
  /** The one `gh` uses by default (`gh auth switch` changes it). */
  active: boolean;
  /** Whether its token works: not once revoked or expired, until signing in again with `gh auth login`. */
  signedIn: boolean;
}

/** What the GitHub CLI offers to connect workspaces with. */
export interface GhStatus {
  /** Where `gh` was found: `null` if it's not installed (or not where expected). */
  path: string | null;
  /** The accounts it knows on github.com, signed in or not anymore: none without `gh`. */
  accounts: GhAccount[];
}

/** GitHub's rate limit for an account, in points: requests cost 1 or 2, for up to 50 pull requests and issues. */
export interface GitHubRateLimit {
  limit: number;
  remaining: number;
  resetAt: Date;
}

/** A workspace's GitHub connection, as GitHub sees it. */
export interface GitHubApiStatus {
  /** The connected account's rate limit, if GitHub could be asked. */
  rateLimit?: GitHubRateLimit;
  /** Why GitHub couldn't be asked, if so. */
  problem?: GitHubProblem;
}

/**
 * What a backup was made for:
 * - `manual`: asked for;
 * - `automatic`: made when the workspace is opened, at most once a day (only the most recent ones are kept);
 * - `pre_migration`: made before migrating its database to a newer version of todai;
 * - `pre_restore`: its state before restoring another backup, so that restoring can be undone.
 */
export type WorkspaceBackupKind = "manual" | "automatic" | "pre_migration" | "pre_restore";

/** A backup of a workspace's database, kept in the app's data directory. */
export interface WorkspaceBackup {
  /** UUID v7. */
  id: UUID;
  workspaceId: UUID;
  /** Absolute path of its file. */
  path: string;
  kind: WorkspaceBackupKind;
  /** Whether its file still exists (e.g. not deleted by hand). */
  available: boolean;
  /** Of its file, in bytes, if available. */
  size?: number;
  createdAt: Date;
}

/** Commands about the app itself rather than the active workspace's content. */
export type ClientInvocationAppCommand = {
  /** Sorted by name. */
  list_workspaces: {
    args?: never;
    returns: Workspace[];
  };
  /** If its database could not be opened, it's not `available`: workspace commands then fail until switching. */
  get_active_workspace: {
    args?: never;
    returns: Workspace | null;
  };
  /**
   * Creates a workspace with a new database at `path`, switching to it.
   *
   * Fails if anything but an empty file already exists at `path`.
   */
  create_workspace: {
    args: {
      name: string;
      /** Absolute; its directory is created if needed. */
      path: string;
    };
    returns: Workspace;
  };
  /**
   * Registers a workspace whose database already exists at `path`, switching to it. Its database is migrated if it
   * comes from an older version of todai.
   *
   * Fails if it's not a todai workspace database (blank ones included), comes from a newer version, or is already
   * registered.
   */
  import_workspace: {
    args: {
      name: string;
      /** Absolute. */
      path: string;
    };
    returns: Workspace;
  };
  /** Fails if its database cannot be opened. */
  switch_to_workspace: {
    args: {
      id: UUID;
    };
    returns: Workspace;
  };
  /** Renames it in its database, which must be available. */
  rename_workspace: {
    args: {
      id: UUID;
      name: string;
    };
    returns: Workspace;
  };
  /**
   * Unregisters it, keeping its database: it can be imported again. If it was the active one, there is no active
   * workspace anymore.
   */
  remove_workspace: {
    args: {
      id: UUID;
      /** Whether its backups are deleted too, rather than only unlisted (their files kept). */
      deleteBackups: boolean;
    };
    returns: never;
  };
  /** Most recent first. */
  list_workspace_backups: {
    args: {
      workspaceId: UUID;
    };
    returns: WorkspaceBackup[];
  };
  /** A manual backup. Fails if its database is unavailable. */
  create_workspace_backup: {
    args: {
      workspaceId: UUID;
    };
    returns: WorkspaceBackup;
  };
  /** With its file. */
  delete_workspace_backup: {
    args: {
      id: UUID;
    };
    returns: never;
  };
  /**
   * Writes a copy of its database to `path`, not listed among its backups. A file there is replaced, unless it's a
   * workspace's database or a backup. Fails if its database is unavailable.
   */
  export_workspace: {
    args: {
      workspaceId: UUID;
      /** Absolute, in an existing directory. */
      path: string;
    };
    returns: never;
  };
  /** Opens the directory of its backups in the file manager. */
  open_workspace_backups_folder: {
    args: {
      workspaceId: UUID;
    };
    returns: never;
  };
  /**
   * Restores it into its workspace's database, after backing up its current state as a `pre_restore` backup:
   * restoring the latter undoes this. If its workspace's database is missing, it's recreated from it instead: the
   * workspace is then available again.
   *
   * Fails without changing anything if it can't be restored, or if its workspace's database is there but can't be
   * opened (so neither backed up first).
   */
  restore_workspace_backup: {
    args: {
      id: UUID;
    };
    returns: Workspace;
  };
  /**
   * Creates a workspace named `name` with a copy of it as its database, at `path`, then switches to it. The backup and
   * its workspace are left as they are.
   *
   * Fails if anything but an empty file already exists at `path`, or if it's in the backups directory.
   */
  restore_workspace_backup_as_new: {
    args: {
      id: UUID;
      name: string;
      /** Absolute; its directory is created if needed. */
      path: string;
    };
    returns: Workspace;
  };
  /** Its automatic backups beyond `autoBackupKeep` are deleted right away. */
  set_workspace_auto_backup: {
    args: {
      workspaceId: UUID;
      autoBackup: boolean;
      /** From 1 to 100. */
      autoBackupKeep: number;
    };
    returns: Workspace;
  };
  /** `gh` is looked for once, then remembered: `refind` looks for it again, e.g. after installing it. */
  get_gh_status: {
    args?: {
      refind?: boolean;
    };
    returns: GhStatus;
  };
  /** Instead of any other account. Fails unless `gh` is signed in to `login` on github.com. */
  connect_github: {
    args: {
      workspaceId: UUID;
      login: string;
    };
    returns: Workspace;
  };
  /** Its links then only show what their URL tells. */
  disconnect_github: {
    args: {
      workspaceId: UUID;
    };
    returns: Workspace;
  };
  /**
   * Asks GitHub for the rate limit of the account a workspace is connected to, which also checks that GitHub accepts
   * its token. Fails if it isn't connected.
   */
  get_github_api_status: {
    args: {
      workspaceId: UUID;
    };
    returns: GitHubApiStatus;
  };
  set_workspace_links_behaviors: {
    args: {
      workspaceId: UUID;
      linksAutoRefresh: boolean;
      linksSuggestDone: boolean;
      linksPasteToCreate: boolean;
    };
    returns: Workspace;
  };
};

export const invokeApiClient = createInvoker<ClientInvocationAppCommand>("app command");
