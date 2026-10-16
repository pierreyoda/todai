import type { UUID } from "node:crypto";
import { queryOptions } from "@tanstack/svelte-query";
import {
  refreshIntegrationsAttention,
  workspaceKeys,
} from "#domains/workspaces";
import { queryClient } from "#shared/ipc/queryClient.ts";
import { invokeIntegrations } from "./commands";

/**
 * The GitHub CLI (`gh`) on this machine. Like the recognized links, it doesn't come from the active workspace's
 * database: it's kept when switching to another one.
 */
export const ghKeys = {
  all: ["gh"] as const,
  status: () => [...ghKeys.all, "status"] as const,
};

/**
 * Whether `gh` is installed, with its accounts on github.com. Unlike the app's own data, it changes behind its back
 * (`gh auth login`, `gh auth logout`): refetched each time it's shown.
 */
export const ghStatusQueryOptions = queryOptions({
  queryKey: ghKeys.status(),
  queryFn: () => invokeIntegrations({ name: "get_gh_status" }),
  staleTime: 0,
});

/**
 * What Notion says of a workspace's connection: whose token it is, or why it can't be asked (e.g. the token expired).
 * Unlike the app's own data, it changes behind its back: refetched each time it's shown.
 */
export const notionApiStatusQueryOptions = (workspaceId: UUID) =>
  queryOptions({
    queryKey: workspaceKeys.notion(workspaceId),
    queryFn: async () => {
      const status = await invokeIntegrations({
        name: "get_notion_api_status",
        args: { workspaceId },
      });
      refreshIntegrationsAttention(queryClient);
      return status;
    },
    staleTime: 0,
  });

/**
 * What GitHub says of a workspace's connection: its account's rate limit, or why it can't be asked. Unlike the app's
 * own data, it changes behind its back: refetched each time it's shown.
 */
export const githubApiStatusQueryOptions = (workspaceId: UUID) =>
  queryOptions({
    queryKey: workspaceKeys.github(workspaceId),
    queryFn: async () => {
      const status = await invokeIntegrations({
        name: "get_github_api_status",
        args: { workspaceId },
      });
      refreshIntegrationsAttention(queryClient);
      return status;
    },
    staleTime: 0,
  });
