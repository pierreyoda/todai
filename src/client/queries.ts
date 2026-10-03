import type { UUID } from "node:crypto";
import { QueryClient, queryOptions } from "@tanstack/svelte-query";

import { invokeClient } from ".";
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
  queryFn: () => invokeClient({ name: "list_todo_months", args: {} }),
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
  queryFn: () => invokeClient({ name: "list_tags", args: {} }),
});
