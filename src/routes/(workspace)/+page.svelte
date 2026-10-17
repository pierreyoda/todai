<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { keepProblems, PROVIDERS, type Provider } from "#core/integrations";
import { dateToTodaiDate } from "#core/time";
import type { Tag } from "#domains/tags";
import { tagsQueryOptions } from "#domains/tags";
import {
  linkStatusesQueryOptions,
  TodoPanel,
  todosQueryOptions,
} from "#domains/todos";
import { activeWorkspaceQueryOptions, isConnected } from "#domains/workspaces";
import SidePanel from "#shell/SidePanel.svelte";

// Usable, as the page is only shown then (see `ActiveWorkspaceGuard`)
const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);

const day = dateToTodaiDate(new Date());
const tags = createQuery(() => tagsQueryOptions);
const todos = createQuery(() => todosQueryOptions(day));
let selectedTagId = $state<Tag["id"] | null>(null);

// The statuses of the todos' links, through the workspace's connections to their providers: refreshed as shown, then
// regularly, unless only when asked (e.g. once a link is added)
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
</script>

<main>
  <SidePanel
    tags={tags.isLoading ? "loading" : tags.error ? "error" : dayTags}
    {selectedTagId}
    onSelectedTagChanged={(tagId) => (selectedTagId = tagId)}
  />
  {#if tags.isLoading}
    Loading...
  {:else if tags.error}
    Error
  {:else}
    <TodoPanel
      {day}
      {todos}
      tags={dayTags}
      {selectedTagId}
      onSelectedTagChanged={(id) => (selectedTagId = id)}
      linksProblems={keepProblems(linkStatuses.data?.problems, connectedTo)}
    />
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
