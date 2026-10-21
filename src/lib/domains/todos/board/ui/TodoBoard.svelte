<script lang="ts">
import type { CreateQueryResult } from "@tanstack/svelte-query";
import { type ProviderProblems, problemEntries } from "#core/integrations";
import type { Day } from "#core/time";
import type { Tag } from "#domains/tags";
import SearchField from "#shared/ui/SearchField.svelte";
import { isDefined } from "#shared/utils/isDefined.ts";
import ConnectIntegrationTip from "../../links/ui/ConnectIntegrationTip.svelte";
import LinkStatusesIndicator from "../../links/ui/LinkStatusesIndicator.svelte";
import type { Todo } from "../../model/types";
import { TodoSearch } from "../../state/todoSearch.svelte";
import TodoCreateSection from "../../ui/TodoCreateSection.svelte";
import TodoPanelFilters from "../../ui/TodoPanelFilters.svelte";
import TodosEmptyState from "../../ui/TodosEmptyState.svelte";
import UnfinishedTodosBanner from "../../ui/UnfinishedTodosBanner.svelte";
import { createMoveOnBoardMutation } from "../api/mutations";
import {
  BOARD_COLUMNS,
  type BoardDrop,
  boardColumns,
  planDrop,
} from "../model/board";
import BoardColumn from "./BoardColumn.svelte";

type TodoBoardProps = {
  day: Day;
  /** The todos of `day`, queried by the page, which also derives the tags' counts from them. */
  todos: CreateQueryResult<Todo[]>;
  tags: readonly Tag[];
  selectedTagId: Tag["id"] | null;
  onSelectedTagChanged: (id: Tag["id"] | null) => void;
  /** Why providers couldn't be asked about the statuses of the todos' links, if so. */
  linksProblems?: ProviderProblems;
};
const {
  day,
  todos,
  tags,
  selectedTagId,
  onSelectedTagChanged,
  linksProblems,
}: TodoBoardProps = $props();

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
const columns = $derived(boardColumns(filteredTodos));
/**
 * Not while a search or a tag hides some of them, among which dropped todos could land: they then only change column.
 */
const reorderable = $derived(!search.active && !selectedTagId);

const moveOnBoard = createMoveOnBoardMutation(() => day);
const drop = (id: Todo["id"], boardDrop: BoardDrop) => {
  // Planned among all of the day's todos, as they're stored
  const dayTodos = todos.data ?? [];
  const todo = dayTodos.find((other) => other.id === id);
  const move = planDrop(dayTodos, id, boardDrop, { reorder: reorderable });
  if (todo && move) moveOnBoard.mutate({ todo, move });
};

const filterSelectedTags = $derived(
  [tags.find(({ id }) => id === selectedTagId) ?? null].filter(isDefined),
);
/** Only when the filters are what empty the board: otherwise, the day has no todos at all. */
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
    {#if problemEntries(linksProblems).length > 0}
      <LinkStatusesIndicator
        problems={linksProblems ?? {}}
        links={todos.data?.flatMap((todo) => todo.links) ?? []}
      />
    {/if}
  </div>
  <TodoCreateSection {day} />
  <UnfinishedTodosBanner {day} {tags} />
  {#if todos.isPending}
    <p class="status">Loading…</p>
  {:else if todos.isError}
    <p class="error" role="alert">{String(todos.error)}</p>
  {:else if filteredTodos.length > 0}
    <!-- Scrolls on its own, sideways too when the window is narrow -->
    <div class="board">
      {#each BOARD_COLUMNS as column (column)}
        <BoardColumn
          {column}
          todos={columns[column]}
          {tags}
          {reorderable}
          onDrop={(id, next) => drop(id, { column, next })}
        />
      {/each}
    </div>
    <ConnectIntegrationTip todos={todos.data ?? []} />
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
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    /* Takes the width left by the side panel */
    @apply flex h-full min-w-0 flex-1 flex-col p-4;
  }

  .toolbar {
    @apply flex items-center gap-2 p-2;
  }

  .board {
    /* Columns as tall as the space left, side by side, never narrower than a card */
    @apply grid min-h-0 flex-1 grid-cols-[repeat(4,minmax(15rem,1fr))] gap-3 overflow-auto p-2;
  }

  .status {
    @apply p-2 text-sm text-slate-700;
  }
  .error {
    @apply p-2 text-sm text-red-700;
  }
</style>
