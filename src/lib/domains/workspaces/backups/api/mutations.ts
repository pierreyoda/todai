import {
  createMutation,
  type QueryClient,
  useQueryClient,
} from "@tanstack/svelte-query";

import type { MutationHooks } from "#shared/ipc/mutations.ts";
import { invalidateActiveWorkspace, workspaceKeys } from "../../api/queries";
import type { Workspace } from "../../model/types";
import type { WorkspaceBackup } from "../model/types";
import { invokeBackups } from "./commands";

type BackedUp = Pick<Workspace, "id">;
type Backup = Pick<WorkspaceBackup, "id">;
type Export = { workspace: Pick<Workspace, "id">; path: string };
type RestoringAsNew = { backup: Backup; name: string; path: string };
type AutoBackup = Pick<Workspace, "id" | "autoBackup" | "autoBackupKeep">;

/**
 * Refreshes the backups, nested in the workspaces: their workspace's last backup, in the list, too. Returned by the
 * mutations, so that they stay pending until they're refetched.
 */
const refreshBackups = (queryClient: QueryClient) =>
  queryClient.invalidateQueries({ queryKey: workspaceKeys.all });

/** Backs up a workspace, as a manual backup. Must be called during component initialization. */
export const createBackUpWorkspaceMutation = (
  hooks: MutationHooks<BackedUp, WorkspaceBackup> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id }: BackedUp) =>
      invokeBackups({
        name: "create_workspace_backup",
        args: { workspaceId: id },
      }),
    onSuccess: (backup, workspace) => {
      hooks.onSuccess?.(backup, workspace);
      return refreshBackups(queryClient);
    },
    onError: (error, workspace) => hooks.onError?.(error, workspace),
  }));
};

/** Deletes a backup, with its file. Must be called during component initialization. */
export const createDeleteBackupMutation = (
  hooks: MutationHooks<Backup> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id }: Backup) =>
      invokeBackups({ name: "delete_workspace_backup", args: { id } }),
    onSuccess: (_, backup) => {
      hooks.onSuccess?.(undefined, backup);
      return refreshBackups(queryClient);
    },
    onError: (error, backup) => hooks.onError?.(error, backup),
  }));
};

/**
 * Writes a copy of a workspace's database to `path`, not listed among its backups. Must be called during component
 * initialization.
 */
export const createExportWorkspaceMutation = (
  hooks: MutationHooks<Export> = {},
) =>
  createMutation(() => ({
    mutationFn: ({ workspace, path }: Export) =>
      invokeBackups({
        name: "export_workspace",
        args: { workspaceId: workspace.id, path },
      }),
    onSuccess: (_, exported) => hooks.onSuccess?.(undefined, exported),
    onError: (error, exported) => hooks.onError?.(error, exported),
  }));

/** Opens the folder of a workspace's backups in the file manager. Must be called during component initialization. */
export const createShowBackupsFolderMutation = (
  hooks: MutationHooks<BackedUp> = {},
) =>
  createMutation(() => ({
    mutationFn: ({ id }: BackedUp) =>
      invokeBackups({
        name: "open_workspace_backups_folder",
        args: { workspaceId: id },
      }),
    onSuccess: (_, workspace) => hooks.onSuccess?.(undefined, workspace),
    onError: (error, workspace) => hooks.onError?.(error, workspace),
  }));

/**
 * Restores a backup into its workspace's database, after backing up its current state. Must be called during component
 * initialization.
 */
export const createRestoreBackupMutation = (
  hooks: MutationHooks<Backup, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id }: Backup) =>
      invokeBackups({ name: "restore_workspace_backup", args: { id } }),
    onSuccess: (restored, backup) => {
      hooks.onSuccess?.(restored, backup);
      // Also refreshes the todos and tags, in case it's the active workspace's
      return invalidateActiveWorkspace(queryClient);
    },
    onError: (error, backup) => hooks.onError?.(error, backup),
  }));
};

/**
 * Creates a workspace with a copy of a backup as its database, then switches to it. Must be called during component
 * initialization.
 */
export const createRestoreBackupAsNewMutation = (
  hooks: MutationHooks<RestoringAsNew, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ backup, name, path }: RestoringAsNew) =>
      invokeBackups({
        name: "restore_workspace_backup_as_new",
        args: { id: backup.id, name, path },
      }),
    onSuccess: (restored, restoring) => {
      hooks.onSuccess?.(restored, restoring);
      return invalidateActiveWorkspace(queryClient);
    },
    onError: (error, restoring) => hooks.onError?.(error, restoring),
  }));
};

/**
 * Sets whether a workspace is backed up automatically, and how many of those backups are kept. Must be called during
 * component initialization.
 */
export const createSetAutoBackupMutation = (
  hooks: MutationHooks<AutoBackup, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id, autoBackup, autoBackupKeep }: AutoBackup) =>
      invokeBackups({
        name: "set_workspace_auto_backup",
        args: { workspaceId: id, autoBackup, autoBackupKeep },
      }),
    onSuccess: (updated, settings) => hooks.onSuccess?.(updated, settings),
    onError: (error, settings) => hooks.onError?.(error, settings),
    // Even on failure, so that the fields show the saved settings again. Also refreshes its backups, maybe pruned.
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
  }));
};
