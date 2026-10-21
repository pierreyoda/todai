<script lang="ts">
import { dateToTodaiDate } from "#core/time";
import type { Tag } from "#domains/tags";
import { createDayTodos, TodoPanel } from "#domains/todos";
import SidePanel from "#shell/SidePanel.svelte";

const day = dateToTodaiDate(new Date());
// With the tags counting them, and their links' statuses
const dayTodos = createDayTodos(day);
const { todos, tags } = dayTodos;
let selectedTagId = $state<Tag["id"] | null>(null);
</script>

<main>
  <SidePanel
    tags={tags.isLoading ? "loading" : tags.error ? "error" : dayTodos.dayTags}
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
      tags={dayTodos.dayTags}
      {selectedTagId}
      onSelectedTagChanged={(id) => (selectedTagId = id)}
      linksProblems={dayTodos.linksProblems}
    />
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
