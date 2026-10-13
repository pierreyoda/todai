import { type QueryClient, queryOptions } from "@tanstack/svelte-query";

import type { Day, Month } from "#core/time";
import { addDays, monthBounds, monthOf } from "#core/time";
import { workspaceDataKey } from "#shared/ipc/queryClient.ts";
import { UNFINISHED_TODOS_LOOKBACK_DAYS } from "../model/constants";
import type { Todo } from "../model/types";
import { invokeTodos } from "./commands";

/**
 * Days are nested in their month, so that invalidating a month also refreshes the lists of its days.
 * Month keys are `YYYY-MM`, so they never collide with `"months"`.
 */
export const todoKeys = {
  all: [...workspaceDataKey, "todos"] as const,
  /** Per-month statistics. */
  months: () => [...todoKeys.all, "months"] as const,
  month: (month: Month) => [...todoKeys.all, month] as const,
  day: (day: Day) => [...todoKeys.month(monthOf(day)), day] as const,
  /** Unfinished todos of the days before one, which may span several months: refreshed along with any of them. */
  unfinished: () => [...todoKeys.all, "unfinished"] as const,
  unfinishedBefore: (day: Day) => [...todoKeys.unfinished(), day] as const,
};

export const todosQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: todoKeys.day(day),
    queryFn: () => invokeTodos({ name: "list_todos", args: { day } }),
  });

/** All the todos of `month`, by day then in display order. */
export const monthTodosQueryOptions = (month: Month) =>
  queryOptions({
    queryKey: todoKeys.month(month),
    queryFn: () => {
      const [start, end] = monthBounds(month);
      return invokeTodos({ name: "list_todos_between", args: { start, end } });
    },
  });

export const todoMonthsQueryOptions = queryOptions({
  queryKey: todoKeys.months(),
  queryFn: () => invokeTodos({ name: "list_todo_months" }),
});

/** The todos left unfinished in the `UNFINISHED_TODOS_LOOKBACK_DAYS` days before `day`, by day then in display order. */
export const unfinishedTodosQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: todoKeys.unfinishedBefore(day),
    queryFn: async () => {
      const todos = await invokeTodos({
        name: "list_todos_between",
        args: {
          start: addDays(day, -UNFINISHED_TODOS_LOOKBACK_DAYS),
          end: addDays(day, -1),
        },
      });
      return todos.filter((todo) => !todo.completed);
    },
  });

/**
 * Refreshes what depends on the todos of `day`, after changing them: its month (with its days' lists), the per-month
 * statistics, and the unfinished todos before a day. Awaitable, so that a mutation can stay pending until then.
 */
export const invalidateTodosOf = (queryClient: QueryClient, day: Day) =>
  Promise.all([
    queryClient.invalidateQueries({ queryKey: todoKeys.month(monthOf(day)) }),
    queryClient.invalidateQueries({ queryKey: todoKeys.months() }),
    queryClient.invalidateQueries({ queryKey: todoKeys.unfinished() }),
  ]);

/** The cached lists that can hold a todo of `day`: its month, with its days' lists, and the unfinished todos. */
const listsKeysOf = (day: Day) => [
  todoKeys.month(monthOf(day)),
  todoKeys.unfinished(),
];

/**
 * Cancels the refetches in flight of the lists holding `todo`, so that they don't overwrite a change made to them in
 * the cache.
 */
export const cancelTodoListsOf = (
  queryClient: QueryClient,
  todo: Pick<Todo, "day">,
) =>
  Promise.all(
    listsKeysOf(todo.day).map((queryKey) =>
      queryClient.cancelQueries({ queryKey }),
    ),
  );

/**
 * Sets the description of `todo` (`undefined` for none) in the cached lists holding it, rather than refetching them:
 * e.g. at each autosave. Lists without it are left as they are.
 */
export const setCachedTodoDescription = (
  queryClient: QueryClient,
  todo: Pick<Todo, "id" | "day">,
  description: string | undefined,
) => {
  for (const queryKey of listsKeysOf(todo.day)) {
    queryClient.setQueriesData<Todo[]>({ queryKey }, (todos) =>
      todos?.some(({ id }) => id === todo.id)
        ? todos.map((cached) =>
            cached.id === todo.id ? { ...cached, description } : cached,
          )
        : todos,
    );
  }
};
