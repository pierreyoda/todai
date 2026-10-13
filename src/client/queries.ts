import type { UUID } from "node:crypto";
import { type QueryClient, queryOptions } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import { invokeApiClient } from "./app";
import type { Day, Month } from "#core/time";
import type { Todo } from "./types";
import { addDays, monthBounds, monthOf } from "#core/time";
import { UNFINISHED_TODOS_LOOKBACK_DAYS } from "../constants";
import { queryClient, workspaceDataKey } from "#shared/ipc/queryClient.ts";
import { refreshGitHubAttention, workspaceKeys } from "#domains/workspaces";

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

/** The todos left unfinished in the `UNFINISHED_TODOS_LOOKBACK_DAYS` days before `day`, by day then in display order. */
export const unfinishedTodosQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: todoKeys.unfinishedBefore(day),
    queryFn: async () => {
      const todos = await invokeClient({
        name: "list_todos_between",
        args: { start: addDays(day, -UNFINISHED_TODOS_LOOKBACK_DAYS), end: addDays(day, -1) },
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
const listsKeysOf = (day: Day) => [todoKeys.month(monthOf(day)), todoKeys.unfinished()];

/**
 * Cancels the refetches in flight of the lists holding `todo`, so that they don't overwrite a change made to them in
 * the cache.
 */
export const cancelTodoListsOf = (queryClient: QueryClient, todo: Pick<Todo, "day">) =>
  Promise.all(listsKeysOf(todo.day).map((queryKey) => queryClient.cancelQueries({ queryKey })));

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
        ? todos.map((cached) => (cached.id === todo.id ? { ...cached, description } : cached))
        : todos,
    );
  }
};


/**
 * The refreshes of the GitHub links' statuses, by day: what the last one ran into. Like the todos, they come from the
 * active workspace's database: refreshed when switching to another one.
 */
export const linkStatusKeys = {
  all: [...workspaceDataKey, "linkStatuses"] as const,
  day: (day: Day) => [...linkStatusKeys.all, day] as const,
  /** Days are `YYYY-MM-DD`, so they never collide with `"preview"`. */
  preview: (url: string) => [...linkStatusKeys.all, "preview", url] as const,
};

/** How often the GitHub links' statuses are refreshed while shown. Only the ones fetched long enough ago are. */
const LINK_STATUSES_INTERVAL = 10 * 60 * 1000;

/**
 * Refreshes the status of the GitHub links of the todos of `day` that were fetched long enough ago, then the todos if
 * any was. Unlike the app's own data, it changes behind its back: each time it's shown, every 10 minutes, and on window
 * focus.
 */
export const linkStatusesQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: linkStatusKeys.day(day),
    queryFn: async () => {
      const outcome = await invokeClient({ name: "refresh_links", args: { day } });
      refreshGitHubAttention(queryClient);
      if (outcome.refreshed > 0) await invalidateTodosOf(queryClient, day);
      return outcome;
    },
    staleTime: 0,
    refetchInterval: LINK_STATUSES_INTERVAL,
    refetchOnWindowFocus: true,
  });

/**
 * Refreshes the statuses of the GitHub links of the todos of `day` that are due, whether they're refreshed on their own
 * or not (e.g. once one is added). Resolves once done, even if it fails.
 */
export const refreshLinksOf = (queryClient: QueryClient, day: Day) =>
  queryClient.fetchQuery({ ...linkStatusesQueryOptions(day), staleTime: 0 }).catch(() => undefined);

/**
 * What `url` would show once added: what GitHub tells about its pull request or issue, through the active workspace's
 * connection. Kept a minute, as it's asked for while typing.
 */
export const linkPreviewQueryOptions = (url: string) =>
  queryOptions({
    queryKey: linkStatusKeys.preview(url),
    queryFn: async () => {
      const preview = await invokeClient({ name: "preview_link", args: { url } });
      refreshGitHubAttention(queryClient);
      return preview;
    },
    staleTime: 60 * 1000,
  });

/**
 * What URLs are recognized as, by URL. Unlike the todos' links, they don't come from the active workspace's database:
 * they're kept when switching to another one.
 */
export const linkKeys = {
  all: ["links"] as const,
  parse: (url: string) => [...linkKeys.all, "parse", url] as const,
};

/** What `url` is recognized as, if it's an http(s) URL: fails otherwise. */
export const parseLinkQueryOptions = (url: string) =>
  queryOptions({
    queryKey: linkKeys.parse(url),
    queryFn: () => invokeClient({ name: "parse_link", args: { url } }),
  });

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
