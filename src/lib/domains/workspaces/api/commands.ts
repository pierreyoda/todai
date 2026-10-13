import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Workspace } from "../model/types";

type WorkspaceCommands = {
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
};

export const invokeWorkspaces = createInvoker<WorkspaceCommands>("app command");
