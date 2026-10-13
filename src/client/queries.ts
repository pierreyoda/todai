import type { UUID } from "node:crypto";
import { queryOptions } from "@tanstack/svelte-query";

import { invokeApiClient } from "./app";
import { queryClient } from "#shared/ipc/queryClient.ts";
import { refreshGitHubAttention, workspaceKeys } from "#domains/workspaces";

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
  queryFn: () => invokeApiClient({ name: "get_gh_status" }),
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
      const status = await invokeApiClient({ name: "get_github_api_status", args: { workspaceId } });
      refreshGitHubAttention(queryClient);
      return status;
    },
    staleTime: 0,
  });
