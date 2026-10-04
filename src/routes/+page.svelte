<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import type { Tag } from "../client/types";
  import { dateToTodaiDate } from "../utils/dates";
  import { tagsQueryOptions, todosQueryOptions } from "../client/queries";
  import SidePanel from "../components/SidePanel.svelte";
  import TodoPanel from "../components/todos/TodoPanel.svelte";

  const day = dateToTodaiDate(new Date());
  const tags = createQuery(() => tagsQueryOptions);
  const todos = createQuery(() => todosQueryOptions(day));
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
    <TodoPanel {day} {todos} tags={dayTags} {selectedTagId} />
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
