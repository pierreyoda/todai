import type { UUID } from "node:crypto";
import { type QueryClient, queryOptions } from "@tanstack/svelte-query";

import { queryClient, workspaceDataKey } from "#shared/ipc/queryClient.ts";
import { invokeWorkspaces } from "./commands";

/**
 * The active workspace and each workspace's backups are nested in the list, so that invalidating the list also
 * refreshes them (e.g. a workspace's last backup, shown in the list). Ids are UUIDs, so they never collide with
 * `"active"`.
 */
export const workspaceKeys = {
  all: ["workspaces"] as const,
  active: () => [...workspaceKeys.all, "active"] as const,
  id: (id: UUID) => [...workspaceKeys.all, id] as const,
  backups: (id: UUID) => [...workspaceKeys.id(id), "backups"] as const,
  github: (id: UUID) => [...workspaceKeys.id(id), "github"] as const,
};

/**
 * Refreshes the workspaces once a provider was asked: they tell whether their connection to it needs the user (e.g.
 * `githubAttention`). Only them, not what's nested in their keys (e.g. their GitHub API status, which may have just been
 * asked).
 */
export const refreshIntegrationsAttention = (queryClient: QueryClient) =>
  Promise.all([
    queryClient.invalidateQueries({ queryKey: workspaceKeys.all, exact: true }),
    queryClient.invalidateQueries({
      queryKey: workspaceKeys.active(),
      exact: true,
    }),
  ]);

/**
 * Refreshes the workspaces (with the active one) and, as they come from its database, the todos, tags, notes and link
 * statuses: after switching to another workspace. Awaitable, so that a mutation can stay pending until then.
 */
export const invalidateActiveWorkspace = (queryClient: QueryClient) =>
  Promise.all([
    queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
    queryClient.invalidateQueries({ queryKey: workspaceDataKey }),
  ]);

/**
 * Refreshes the workspaces and drops the todos, tags, notes and link statuses, which came from the active workspace's
 * database: after closing it without opening another one, so there is nothing to refetch them from. Awaitable, so that a
 * mutation can stay pending until then.
 */
export const clearActiveWorkspace = (queryClient: QueryClient) => {
  queryClient.removeQueries({ queryKey: workspaceDataKey });
  return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
};

/** Sorted by name. */
export const workspacesQueryOptions = queryOptions({
  queryKey: workspaceKeys.all,
  queryFn: () => invokeWorkspaces({ name: "list_workspaces" }),
});

export const activeWorkspaceQueryOptions = queryOptions({
  queryKey: workspaceKeys.active(),
  queryFn: () => invokeWorkspaces({ name: "get_active_workspace" }),
});

/** Found in the list, as there is no command for a single workspace; `null` if there is none with `id`. */
export const workspaceQueryOptions = (id: UUID) =>
  queryOptions({
    queryKey: workspaceKeys.id(id),
    queryFn: async () => {
      const workspaces = await queryClient.query(workspacesQueryOptions);
      return workspaces.find((workspace) => workspace.id === id) ?? null;
    },
  });
