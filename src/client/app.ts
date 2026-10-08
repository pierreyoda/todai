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
};

export type ClientAppCommandName = keyof ClientInvocationAppCommand;

export const invokeApiClient = async <N extends ClientAppCommandName, R = ClientInvocationAppCommand[N]["returns"]>(c: Invocation<ClientInvocationAppCommand, N>): Promise<R> =>
  invoke(c.name, c.args).then(parseResponse<R>).catch((e) => {
    console.error(`Error invoking app command ${c.name}:`, e);
    throw e;
  });
