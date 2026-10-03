<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { todosQueryOptions } from "../../client/queries";
  import type { Day, Tag } from "../../client/types";
  import TodoCreateSection from "./TodoCreateSection.svelte";
  import TodoList from "./TodoList.svelte";

  type TodoPanelProps = {
    day: Day;
    tags: readonly Tag[];
  };

  const { day, tags }: TodoPanelProps = $props();

  const todos = createQuery(() => todosQueryOptions(day));
</script>

<section class="container">
  <TodoCreateSection {day} />
  {#if todos.isPending}
    <p class="status">Loading…</p>
  {:else if todos.isError}
    <p class="error" role="alert">{String(todos.error)}</p>
  {:else}
    <TodoList todos={todos.data} {tags} />
  {/if}
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full h-full flex flex-col p-4;
  }

  .status {
    @apply p-2 text-sm text-slate-700;
  }
  .error {
    @apply p-2 text-sm text-red-700;
  }
</style>
