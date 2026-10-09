<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";
  import type { Tag } from "../client/types";
  import { dateToTodaiDate } from "../utils/dates";
  import {
    activeWorkspaceQueryOptions,
    tagsQueryOptions,
    todosQueryOptions,
  } from "../client/queries";
  import LoadingState from "../components/common/LoadingState.svelte";
  import SidePanel from "../components/SidePanel.svelte";
  import TodoPanel from "../components/todos/TodoPanel.svelte";

  // Without a usable active workspace (none yet, removed, or its database could not be opened), the user picks or
  // creates one. Until then, there is no database to query the todos and tags from.
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const ready = $derived(activeWorkspace.data?.available === true);
  $effect(() => {
    if (activeWorkspace.isSuccess && !ready) {
      goto("/settings/workspace");
    }
  });

  const day = dateToTodaiDate(new Date());
  const tags = createQuery(() => ({ ...tagsQueryOptions, enabled: ready }));
  const todos = createQuery(() => ({
    ...todosQueryOptions(day),
    enabled: ready,
  }));
  let selectedTagId = $state<Tag["id"] | null>(null);

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
  {#if ready}
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
      />
    {/if}
  {:else}
    <LoadingState state={activeWorkspace.isError ? "error" : "loading"}>
      {#snippet loading()}
        Loading the Workspaces...
      {/snippet}
      {#snippet error()}
        Could not load the Workspaces.
      {/snippet}
    </LoadingState>
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
