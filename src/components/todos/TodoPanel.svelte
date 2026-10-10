<script lang="ts">
  import type { CreateQueryResult } from "@tanstack/svelte-query";

  import type { Day, Tag, Todo } from "../../client/types";
  import SearchField from "../common/SearchField.svelte";
  import TodoList from "./TodoList.svelte";
  import TodoCreateSection from "./TodoCreateSection.svelte";
  import TodoPanelFilters from "./TodoPanelFilters.svelte";
  import TodosEmptyState from "./TodosEmptyState.svelte";
  import { isDefined } from "../../utils";
  import { TodoSearch } from "../../utils/todoSearch.svelte";

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

  const search = new TodoSearch(
    () => todos.data ?? [],
    () => tags,
  );
  const filteredTodos = $derived(
    search.filter(
      selectedTagId
        ? (todos.data?.filter(({ tagIds }) => tagIds.includes(selectedTagId)) ??
            [])
        : (todos.data ?? []),
    ),
  );

  const filterSelectedTags = $derived(
    [tags.find(({ id }) => id === selectedTagId) ?? null].filter(isDefined),
  );
  /** Only when the filters are what empty the list: otherwise, the day has no todos at all. */
  const hasTodos = $derived((todos.data?.length ?? 0) > 0);
  const emptyFilterTag = $derived(
    hasTodos ? (filterSelectedTags[0] ?? null) : null,
  );
  const emptySearchQuery = $derived(
    hasTodos && search.active ? search.query.trim() : null,
  );
</script>

<section class="container">
  <div class="toolbar">
    <SearchField
      bind:value={search.query}
      label="Search todos"
      focusShortcut
      class="max-w-xs"
    />
    <TodoPanelFilters
      selectedTags={filterSelectedTags}
      onDismissedTag={(id) => {
        if (id === selectedTagId) onSelectedTagChanged(null);
      }}
    />
  </div>
  <TodoCreateSection {day} />
  {#if todos.isPending}
    <p class="status">Loading…</p>
  {:else if todos.isError}
    <p class="error" role="alert">{String(todos.error)}</p>
  {:else}
    {#if filteredTodos.length > 0}
      <TodoList todos={filteredTodos} {tags} />
    {:else}
      <!-- Keyed by the case, so switching between cases replays the entrance, but typing a search doesn't -->
      {#key `${emptySearchQuery !== null}:${emptyFilterTag?.id}`}
        <TodosEmptyState
          filterTag={emptyFilterTag}
          onClearFilter={() => onSelectedTagChanged(null)}
          searchQuery={emptySearchQuery}
          onClearSearch={() => search.clear()}
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

  .toolbar {
    @apply flex items-center gap-2 p-2;
  }

  .status {
    @apply p-2 text-sm text-slate-700;
  }
  .error {
    @apply p-2 text-sm text-red-700;
  }
</style>
