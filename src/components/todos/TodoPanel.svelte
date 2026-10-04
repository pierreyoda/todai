<script lang="ts">
  import type { CreateQueryResult } from "@tanstack/svelte-query";

  import type { Day, Tag, Todo } from "../../client/types";
  import TodoCreateSection from "./TodoCreateSection.svelte";
  import TodoList from "./TodoList.svelte";

  type TodoPanelProps = {
    day: Day;
    /** The todos of `day`, queried by the page, which also derives the tags' counts from them. */
    todos: CreateQueryResult<Todo[]>;
    tags: readonly Tag[];
    selectedTagId: Tag["id"] | null;
  };

  const { day, todos, tags, selectedTagId }: TodoPanelProps = $props();

  const filteredTodos = $derived(
    selectedTagId
      ? (todos.data?.filter(({ tagIds }) => tagIds.includes(selectedTagId)) ??
          [])
      : [...(todos.data ?? [])],
  );
</script>

<section class="container">
  <TodoCreateSection {day} />
  {#if todos.isPending}
    <p class="status">Loading…</p>
  {:else if todos.isError}
    <p class="error" role="alert">{String(todos.error)}</p>
  {:else}
    <TodoList todos={filteredTodos} {tags} />
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
