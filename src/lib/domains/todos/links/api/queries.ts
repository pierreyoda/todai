import { type QueryClient, queryOptions } from "@tanstack/svelte-query";

import type { Day } from "#core/time";
import { refreshIntegrationsAttention } from "#domains/workspaces";
import { queryClient, workspaceDataKey } from "#shared/ipc/queryClient.ts";
import { invalidateTodosOf } from "../../api/queries";
import { invokeLinks, invokeNotionRules } from "./commands";

/**
 * The refreshes of the links' statuses, by day: what the last one ran into. Like the todos, they come from the active
 * workspace's database: refreshed when switching to another one.
 */
export const linkStatusKeys = {
  all: [...workspaceDataKey, "linkStatuses"] as const,
  day: (day: Day) => [...linkStatusKeys.all, day] as const,
  /** Days are `YYYY-MM-DD`, so they never collide with `"preview"`. */
  preview: (url: string) => [...linkStatusKeys.all, "preview", url] as const,
};

/** How often the links' statuses are refreshed while shown. Only the ones fetched long enough ago are. */
const LINK_STATUSES_INTERVAL = 10 * 60 * 1000;

/**
 * Refreshes the status of the links of the todos of `day` that were fetched long enough ago, then the todos if any was.
 * Unlike the app's own data, it changes behind its back: each time it's shown, every 10 minutes, and on window focus.
 */
export const linkStatusesQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: linkStatusKeys.day(day),
    queryFn: async () => {
      const outcome = await invokeLinks({
        name: "refresh_links",
        args: { day },
      });
      refreshIntegrationsAttention(queryClient);
      if (outcome.refreshed > 0) await invalidateTodosOf(queryClient, day);
      return outcome;
    },
    staleTime: 0,
    refetchInterval: LINK_STATUSES_INTERVAL,
    refetchOnWindowFocus: true,
  });

/**
 * Refreshes the statuses of the links of the todos of `day` that are due, whether they're refreshed on their own or not
 * (e.g. once one is added). Resolves once done, even if it fails.
 */
export const refreshLinksOf = (queryClient: QueryClient, day: Day) =>
  queryClient
    .fetchQuery({ ...linkStatusesQueryOptions(day), staleTime: 0 })
    .catch(() => undefined);

/**
 * What `url` would show once added: what its provider tells about its item (e.g. GitHub about a pull request), through
 * the active workspace's connection. Kept a minute, as it's asked for while typing.
 */
export const linkPreviewQueryOptions = (url: string) =>
  queryOptions({
    queryKey: linkStatusKeys.preview(url),
    queryFn: async () => {
      const preview = await invokeLinks({
        name: "preview_link",
        args: { url },
      });
      refreshIntegrationsAttention(queryClient);
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
    queryFn: () => invokeLinks({ name: "parse_link", args: { url } }),
  });

/** What `url` would show once added (see `linkPreviewQueryOptions`), fetched now: e.g. to title a todo after it. */
export const fetchLinkPreview = (url: string) =>
  queryClient.fetchQuery(linkPreviewQueryOptions(url));

/** What `url` is recognized as (see `parseLinkQueryOptions`), fetched now: e.g. once submitted, before recognized. */
export const fetchParsedLink = (url: string) =>
  queryClient.fetchQuery(parseLinkQueryOptions(url));

/**
 * The done rules of the active workspace's Notion databases, and what Notion tells of their columns. Like the todos,
 * they come from the active workspace: refreshed when switching to another one.
 */
export const notionDoneRuleKeys = {
  all: [...workspaceDataKey, "notionDoneRules"] as const,
  list: () => [...notionDoneRuleKeys.all, "list"] as const,
  columns: (dataSourceId: string) =>
    [...notionDoneRuleKeys.all, "columns", dataSourceId] as const,
};

export const notionDoneRulesQueryOptions = queryOptions({
  queryKey: notionDoneRuleKeys.list(),
  queryFn: () => invokeNotionRules({ name: "list_notion_done_rules" }),
});

/** A database's columns that can tell its pages are done: asked to Notion each time they're shown, e.g. once added. */
export const notionColumnsQueryOptions = (dataSourceId: string) =>
  queryOptions({
    queryKey: notionDoneRuleKeys.columns(dataSourceId),
    queryFn: () =>
      invokeNotionRules({ name: "get_notion_columns", args: { dataSourceId } }),
    staleTime: 0,
  });
