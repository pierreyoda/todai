import { createMutation, useQueryClient } from "@tanstack/svelte-query";
import { linkStatusKeys } from "#domains/todos";
import type { Workspace } from "#domains/workspaces";
import { workspaceKeys } from "#domains/workspaces";
import type { MutationHooks } from "#shared/ipc/mutations.ts";
import { invokeIntegrations } from "./commands";
import { ghKeys } from "./queries";

type LinksBehaviors = Pick<
  Workspace,
  "id" | "linksAutoRefresh" | "linksSuggestDone" | "linksPasteToCreate"
>;

/**
 * Connects a workspace to GitHub through a GitHub CLI account, instead of any other one. Must be called during
 * component initialization.
 */
export const createConnectGitHubMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({
      workspace,
      login,
    }: {
      workspace: Workspace;
      login: string;
    }) =>
      invokeIntegrations({
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

/**
 * Connects a workspace to Notion with a personal access token, instead of any other one. Succeeds with why it wasn't
 * connected when Notion couldn't vouch for the token (e.g. it refused it): nothing is kept then. Must be called during
 * component initialization.
 */
export const createConnectNotionMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({
      workspace,
      token,
    }: {
      workspace: Workspace;
      token: string;
    }) =>
      invokeIntegrations({
        name: "connect_notion",
        args: { workspaceId: workspace.id, token },
      }),
    onSuccess: (outcome) => {
      if (outcome.kind !== "connected") return;
      // The links' statuses, through the new connection, without waiting for Notion
      queryClient.invalidateQueries({ queryKey: linkStatusKeys.all });
      // Returned so the mutation stays pending until the workspaces are refetched, with the connection.
      return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  }));
};

/** Disconnects a workspace from Notion, deleting its token. Must be called during component initialization. */
export const createDisconnectNotionMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (workspace: Workspace) =>
      invokeIntegrations({
        name: "disconnect_notion",
        args: { workspaceId: workspace.id },
      }),
    // Returned so the mutation stays pending until the workspaces are refetched, without the connection.
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
  }));
};

/** Disconnects a workspace from GitHub. Must be called during component initialization. */
export const createDisconnectGitHubMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (workspace: Workspace) =>
      invokeIntegrations({
        name: "disconnect_github",
        args: { workspaceId: workspace.id },
      }),
    // Returned so the mutation stays pending until the workspaces are refetched, without the connection.
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
  }));
};

/**
 * Looks for the GitHub CLI again rather than where it was found, e.g. once installed or signed in to. Must be called
 * during component initialization.
 */
export const createRefindGhMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () =>
      invokeIntegrations({ name: "get_gh_status", args: { refind: true } }),
    // Replaces the cached status, rather than refetching it without looking for `gh` again.
    onSuccess: (status) => queryClient.setQueryData(ghKeys.status(), status),
  }));
};

/**
 * Sets how a workspace's links behave: refreshed on their own, suggesting to complete their todo, offering to create a
 * todo from a pasted one. Must be called during component initialization.
 */
export const createSetLinksBehaviorsMutation = (
  hooks: MutationHooks<LinksBehaviors, Workspace> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ id, ...behaviors }: LinksBehaviors) =>
      invokeIntegrations({
        name: "set_workspace_links_behaviors",
        args: { workspaceId: id, ...behaviors },
      }),
    onSuccess: (updated, behaviors) => hooks.onSuccess?.(updated, behaviors),
    onError: (error, behaviors) => hooks.onError?.(error, behaviors),
    // Even on failure, so that the fields show the saved settings again
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
  }));
};
