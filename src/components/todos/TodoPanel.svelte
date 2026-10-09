<script lang="ts">
  import type { CreateQueryResult } from "@tanstack/svelte-query";

  import type { Day, Tag, Todo } from "../../client/types";
  import TodoList from "./TodoList.svelte";
  import TodoCreateSection from "./TodoCreateSection.svelte";
  import TodoPanelFilters from "./TodoPanelFilters.svelte";
  import TodosEmptyState from "./TodosEmptyState.svelte";
  import { isDefined } from "../../utils";

  type TodoPanelProps = {
    day: Day;
    /** The todos of `day`, queried by the page, which also derives the tags' counts from them. */
    todos: CreateQueryResult<Todo[]>;
    tags: readonly Tag[];
    selectedTagId: Tag["id"] | null;
    onSelectedTagChanged: (id: Tag["id"] | null) => void;
  };
  const {
    day,
    todos,
    tags,
    selectedTagId,
    onSelectedTagChanged,
  }: TodoPanelProps = $props();

  const filteredTodos = $derived(
    selectedTagId
      ? (todos.data?.filter(({ tagIds }) => tagIds.includes(selectedTagId)) ??
          [])
      : [...(todos.data ?? [])],
  );

  const filterSelectedTags = $derived(
    [tags.find(({ id }) => id === selectedTagId) ?? null].filter(isDefined),
  );
  /** Only when the filter is what empties the list: otherwise, the day has no todos at all. */
  const emptyFilterTag = $derived(
    (todos.data?.length ?? 0) > 0 ? (filterSelectedTags[0] ?? null) : null,
  );
</script>

<section class="container">
  <TodoPanelFilters
    selectedTags={filterSelectedTags}
    onDismissedTag={(id) => {
      if (id === selectedTagId) onSelectedTagChanged(null);
    }}
  />
  <TodoCreateSection {day} />
  {#if todos.isPending}
    <p class="status">Loading…</p>
  {:else if todos.isError}
    <p class="error" role="alert">{String(todos.error)}</p>
  {:else}
    {#if filteredTodos.length > 0}
      <TodoList todos={filteredTodos} {tags} />
    {:else}
      <!-- Keyed, so switching between the two cases replays the entrance -->
      {#key emptyFilterTag?.id}
        <TodosEmptyState
          filterTag={emptyFilterTag}
          onClearFilter={() => onSelectedTagChanged(null)}
        />
      {/key}
    {/if}
  {/if}
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    /* Takes the width left by the side panel */
    @apply min-w-0 flex-1 h-full flex flex-col p-4;
  }

  .status {
    @apply p-2 text-sm text-slate-700;
  }
  .error {
    @apply p-2 text-sm text-red-700;
  }
</style>
