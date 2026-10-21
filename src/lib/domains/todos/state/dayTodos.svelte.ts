import { createQuery } from "@tanstack/svelte-query";
import { keepProblems, PROVIDERS, type Provider } from "#core/integrations";
import type { Day } from "#core/time";
import { type Tag, tagsQueryOptions } from "#domains/tags";
import { activeWorkspaceQueryOptions, isConnected } from "#domains/workspaces";
import { todosQueryOptions } from "../api/queries";
import { linkStatusesQueryOptions } from "../links/api/queries";

/**
 * The todos of `day`, as its pages show them (its list, its board), along with the tags and the statuses of their links.
 * Must be called during component initialization, of a page shown once the active workspace is usable (see
 * `ActiveWorkspaceGuard`).
 *
 * The statuses are asked through the workspace's connections to their providers: as shown, then regularly, unless only
 * when asked (e.g. once a link is added).
 */
export const createDayTodos = (day: Day) => {
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const tags = createQuery(() => tagsQueryOptions);
  const todos = createQuery(() => todosQueryOptions(day));

  const connectedTo = (provider: Provider) =>
    isConnected(activeWorkspace.data, provider);
  const connected = $derived(PROVIDERS.some(connectedTo));
  const autoRefresh = $derived(
    connected && !!activeWorkspace.data?.linksAutoRefresh,
  );
  const linkStatuses = createQuery(() => ({
    ...linkStatusesQueryOptions(day),
    enabled: autoRefresh,
  }));
  // Also when coming back to the app: TanStack Query only watches the page's visibility, which switching apps leaves
  $effect(() => {
    if (!autoRefresh) return;
    const refresh = () => linkStatuses.refetch({ cancelRefetch: false });
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  });

  /** The tags, with `linkedTodosCount` scoped to the todos of `day` (unknown until they're loaded). */
  const dayTags = $derived.by(() => {
    const allTags = tags.data ?? [];
    if (!todos.data) {
      return allTags.map((tag) => ({ ...tag, linkedTodosCount: undefined }));
    }
    const counts = new Map<Tag["id"], number>();
    for (const { tagIds } of todos.data) {
      for (const tagId of tagIds) {
        counts.set(tagId, (counts.get(tagId) ?? 0) + 1);
      }
    }
    return allTags.map((tag) => ({
      ...tag,
      linkedTodosCount: counts.get(tag.id) ?? 0,
    }));
  });
  /** Why providers couldn't be asked about the statuses of the todos' links, if so. */
  const linksProblems = $derived(
    keepProblems(linkStatuses.data?.problems, connectedTo),
  );

  return {
    todos,
    tags,
    get dayTags() {
      return dayTags;
    },
    get linksProblems() {
      return linksProblems;
    },
  };
};
