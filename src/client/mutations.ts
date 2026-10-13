import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import { invokeApiClient } from "./app";
import type { Workspace } from "#domains/workspaces";
import { workspaceKeys } from "#domains/workspaces";
import { ghKeys } from "./queries";
import { linkStatusKeys } from "#domains/todos";

/**
 * Connects a workspace to GitHub through a GitHub CLI account, instead of any other one. Must be called during
 * component initialization.
 */
export const createConnectGitHubMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ workspace, login }: { workspace: Workspace; login: string }) =>
      invokeApiClient({
        name: "connect_github",
        args: { workspaceId: workspace.id, login },
      }),
    onSuccess: () => {
      // The links' statuses, through the new connection, without waiting for GitHub
      queryClient.invalidateQueries({ queryKey: linkStatusKeys.all });
      // Returned so the mutation stays pending until the workspaces are refetched, with the connection.
      return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  }));
};

/** Disconnects a workspace from GitHub. Must be called during component initialization. */
export const createDisconnectGitHubMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (workspace: Workspace) =>
      invokeApiClient({
        name: "disconnect_github",
        args: { workspaceId: workspace.id },
      }),
    // Returned so the mutation stays pending until the workspaces are refetched, without the connection.
    onSuccess: () => queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
  }));
};

/**
 * Looks for the GitHub CLI again rather than where it was found, e.g. once installed or signed in to. Must be called
 * during component initialization.
 */
export const createRefindGhMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () => invokeApiClient({ name: "get_gh_status", args: { refind: true } }),
    // Replaces the cached status, rather than refetching it without looking for `gh` again.
    onSuccess: (status) => queryClient.setQueryData(ghKeys.status(), status),
  }));
};
