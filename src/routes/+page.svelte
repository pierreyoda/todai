<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import type { Tag } from "../client/types";
  import { dateToTodaiDate } from "../utils/dates";
  import { tagsQueryOptions } from "../client/queries";
  import SidePanel from "../components/SidePanel.svelte";
  import TodoPanel from "../components/todos/TodoPanel.svelte";

  const day = dateToTodaiDate(new Date());
  const tags = createQuery(() => tagsQueryOptions);
  let selectedTagId = $state<Tag["id"] | null>(null);
</script>

<main>
  <SidePanel
    tags={tags.isLoading ? "loading" : tags.error ? "error" : (tags.data ?? [])}
    {selectedTagId}
    onSelectedTagChanged={(tagId) => (selectedTagId = tagId)}
  />
  <TodoPanel {day} />
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
