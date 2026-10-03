import type { UUID } from "node:crypto";
import { QueryClient, queryOptions } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import type { Day } from "./types";

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

export const todoKeys = {
  all: ["todos"] as const,
  id: (id: UUID) => [...todoKeys.all, id] as const,
  day: (day: Day) => [...todoKeys.all, day] as const,
};

export const todosQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: todoKeys.day(day),
    queryFn: () => invokeClient({ name: "list_todos", args: { day } }),
  });

export const tagKeys = {
  all: ["tags"] as const,
  id: (id: UUID) => [...tagKeys.all, id] as const,
};

export const tagsQueryOptions = queryOptions({
  queryKey: tagKeys.all,
  queryFn: () => invokeClient({ name: "list_tags", args: {} }),
});
