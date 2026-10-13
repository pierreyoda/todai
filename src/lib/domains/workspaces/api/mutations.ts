import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import type { MutationHooks } from "#shared/ipc/mutations.ts";
import { basename } from "#shared/utils/paths.ts";
import type { Workspace } from "../model/types";
import { invokeWorkspaces } from "./commands";
import {
  clearActiveWorkspace,
  invalidateActiveWorkspace,
  workspaceKeys,
} from "./queries";
import { canSwitchWorkspace, runAfterWorkspaceSwitch } from "./switchHooks";

type NewWorkspace = Pick<Workspace, "name" | "path">;
type Renaming = { workspace: Pick<Workspace, "id">; name: string };
type Removal = {
  workspace: Pick<Workspace, "id" | "isActive">;
  deleteBackups: boolean;
};

/** Creates a workspace with a new database, then switches to it. Must be called during component initialization. */
export const createAddWorkspaceMutation = (
  hooks: MutationHooks<NewWorkspace, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ name, path }: NewWorkspace) =>
      invokeWorkspaces({ name: "create_workspace", args: { name, path } }),
    onSuccess: (created, workspace) => {
      hooks.onSuccess?.(created, workspace);
      // Returned so the mutation stays pending until everything is refetched from its database.
      return invalidateActiveWorkspace(queryClient);
    },
    onError: (error, workspace) => hooks.onError?.(error, workspace),
  }));
};

/**
 * Renames a workspace: only its name can change, as there is no command to move its database. Must be called during
 * component initialization.
 */
export const createRenameWorkspaceMutation = (
  hooks: MutationHooks<Renaming, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ workspace, name }: Renaming) =>
      invokeWorkspaces({
        name: "rename_workspace",
        args: { id: workspace.id, name },
      }),
    onSuccess: (renamed, renaming) => {
      hooks.onSuccess?.(renamed, renaming);
      // Invalidating the list also refreshes the active workspace.
      return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
    onError: (error, renaming) => hooks.onError?.(error, renaming),
  }));
};

/**
 * Registers the workspace whose database is at `path`, then switches to it: named after its file, without the
 * extension. Must be called during component initialization.
 */
export const createImportWorkspaceMutation = (
  hooks: MutationHooks<string, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (path: string) =>
      invokeWorkspaces({
        name: "import_workspace",
        args: { name: basename(path).replace(/(?<=.)\.[^.]*$/, ""), path },
      }),
    onSuccess: (imported, path) => {
      hooks.onSuccess?.(imported, path);
      return invalidateActiveWorkspace(queryClient);
    },
    onError: (error, path) => hooks.onError?.(error, path),
  }));
};

/**
 * Unregisters a workspace, keeping its database, and its backups unless asked otherwise. Removing the active one
 * leaves none active: its data is dropped, until the user picks or creates another one. Must be called during component
 * initialization.
 */
export const createRemoveWorkspaceMutation = (
  hooks: MutationHooks<Removal> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ workspace, deleteBackups }: Removal) =>
      invokeWorkspaces({
        name: "remove_workspace",
        args: { id: workspace.id, deleteBackups },
      }),
    onSuccess: (_, removal) => {
      hooks.onSuccess?.(undefined, removal);
      return removal.workspace.isActive
        ? clearActiveWorkspace(queryClient)
        : queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
    onError: (error, removal) => hooks.onError?.(error, removal),
  }));
};

/**
 * Switches to another workspace, once what's being edited in the current one is saved (see `beforeWorkspaceSwitch`):
 * resolves to whether it did. Must be called during component initialization.
 */
export const createSwitchWorkspaceMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: async ({ id }: Workspace) => {
      if (!(await canSwitchWorkspace())) return false;
      await invokeWorkspaces({ name: "switch_to_workspace", args: { id } });
      return true;
    },
    // Returned so the mutation stays pending until everything is refetched from the other workspace's database.
    onSuccess: (switched) => {
      if (!switched) return;
      return Promise.all([
        runAfterWorkspaceSwitch(),
        invalidateActiveWorkspace(queryClient),
      ]);
    },
  }));
};
