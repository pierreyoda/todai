import type { UUID } from "node:crypto";

import { invoke } from "@tauri-apps/api/core";
import { parseResponse } from "../utils/client";
import type { Invocation } from ".";

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
};

export type ClientAppCommandName = keyof ClientInvocationAppCommand;

export const invokeApiClient = async <N extends ClientAppCommandName, R = ClientInvocationAppCommand[N]["returns"]>(c: Invocation<ClientInvocationAppCommand, N>): Promise<R> =>
  invoke(c.name, c.args).then(parseResponse<R>).catch((e) => {
    console.error(`Error invoking app command ${c.name}:`, e);
    throw e;
  });
