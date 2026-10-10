import type { UUID } from "node:crypto";
import { QueryClient, queryOptions } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import { invokeApiClient } from "./app";
import type { Day, Month } from "./types";
import { monthBounds, monthOf } from "../utils/dates";

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
 * Days are nested in their month, so that invalidating a month also refreshes the lists of its days.
 * Month keys are `YYYY-MM`, so they never collide with `"months"`.
 */
export const todoKeys = {
  all: ["todos"] as const,
  /** Per-month statistics. */
  months: () => [...todoKeys.all, "months"] as const,
  month: (month: Month) => [...todoKeys.all, month] as const,
  day: (day: Day) => [...todoKeys.month(monthOf(day)), day] as const,
};

export const todosQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: todoKeys.day(day),
    queryFn: () => invokeClient({ name: "list_todos", args: { day } }),
  });

/** All the todos of `month`, by day then in display order. */
export const monthTodosQueryOptions = (month: Month) =>
  queryOptions({
    queryKey: todoKeys.month(month),
    queryFn: () => {
      const [start, end] = monthBounds(month);
      return invokeClient({ name: "list_todos_between", args: { start, end } });
    },
  });

export const todoMonthsQueryOptions = queryOptions({
  queryKey: todoKeys.months(),
  queryFn: () => invokeClient({ name: "list_todo_months" }),
});

/**
 * Refreshes what depends on the todos of `day`, after changing them: its month (with its days' lists) and the
 * per-month statistics. Awaitable, so that a mutation can stay pending until then.
 */
export const invalidateTodosOf = (queryClient: QueryClient, day: Day) =>
  Promise.all([
    queryClient.invalidateQueries({ queryKey: todoKeys.month(monthOf(day)) }),
    queryClient.invalidateQueries({ queryKey: todoKeys.months() }),
  ]);

export const tagKeys = {
  all: ["tags"] as const,
  id: (id: UUID) => [...tagKeys.all, id] as const,
};

export const tagsQueryOptions = queryOptions({
  queryKey: tagKeys.all,
  queryFn: () => invokeClient({ name: "list_tags" }),
});

export const noteKeys = {
  all: ["notes"] as const,
  day: (day: Day) => [...noteKeys.all, day] as const,
};

/** `null` if `day` has no note. */
export const noteQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: noteKeys.day(day),
    queryFn: () => invokeClient({ name: "get_note", args: { day } }),
  });

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
};

/**
 * Refreshes the workspaces (with the active one) and, as they come from its database, the todos, tags and notes: after
 * switching to another workspace. Awaitable, so that a mutation can stay pending until then.
 */
export const invalidateActiveWorkspace = (queryClient: QueryClient) =>
  Promise.all([
    queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
    queryClient.invalidateQueries({ queryKey: todoKeys.all }),
    queryClient.invalidateQueries({ queryKey: tagKeys.all }),
    queryClient.invalidateQueries({ queryKey: noteKeys.all }),
  ]);

/**
 * Refreshes the workspaces and drops the todos, tags and notes, which came from the active workspace's database: after
 * closing it without opening another one, so there is nothing to refetch them from. Awaitable, so that a mutation can
 * stay pending until then.
 */
export const clearActiveWorkspace = (queryClient: QueryClient) => {
  queryClient.removeQueries({ queryKey: todoKeys.all });
  queryClient.removeQueries({ queryKey: tagKeys.all });
  queryClient.removeQueries({ queryKey: noteKeys.all });
  return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
};

/** Sorted by name. */
export const workspacesQueryOptions = queryOptions({
  queryKey: workspaceKeys.all,
  queryFn: () => invokeApiClient({ name: "list_workspaces" }),
});

export const activeWorkspaceQueryOptions = queryOptions({
  queryKey: workspaceKeys.active(),
  queryFn: () => invokeApiClient({ name: "get_active_workspace" }),
});

/** Most recent first. */
export const workspaceBackupsQueryOptions = (workspaceId: UUID) =>
  queryOptions({
    queryKey: workspaceKeys.backups(workspaceId),
    queryFn: () => invokeApiClient({ name: "list_workspace_backups", args: { workspaceId } }),
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
