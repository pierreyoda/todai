<script lang="ts">
import { dateToTodaiDate } from "#core/time";
import type { Tag } from "#domains/tags";
import { createDayTodos, TodoBoard } from "#domains/todos";
import SidePanel from "#shell/SidePanel.svelte";

// Fixed once opened, like the list's: left open past midnight, it's still the board of the day it was opened
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
    <TodoBoard
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
    @apply flex h-full w-full items-start;
  }
</style>
