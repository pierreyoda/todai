import { QueryClient } from "@tanstack/svelte-query";

/**
 * Data comes from the local Rust backend through IPC, not over the network:
 * - `networkMode: "always"`, as the default ("online") pauses everything while the OS reports being offline;
 * - Rust is the single writer, so data stays fresh until explicitly invalidated;
 * - a failed invocation is a bug to surface, not a flaky request to retry.
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      networkMode: "always",
      staleTime: Infinity,
      refetchOnWindowFocus: false,
      retry: false,
    },
    mutations: {
      networkMode: "always",
    },
  },
});

/**
 * Where the data of the active workspace's database (e.g. its todos) is cached, apart from the app's own (e.g. the
 * workspaces): so that it can be refreshed, or dropped, at once when the active workspace changes.
 */
export const workspaceDataKey = ["workspaceData"] as const;
