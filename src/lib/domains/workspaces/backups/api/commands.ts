import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Workspace } from "../../model/types";
import type { WorkspaceBackup } from "../model/types";

type BackupCommands = {
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
};

export const invokeBackups = createInvoker<BackupCommands>("app command");
