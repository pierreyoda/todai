<script lang="ts">
import { attachDroppable } from "@thisux/sveltednd";
import type { Tag } from "#domains/tags";
import { formatEstimates } from "../../model/estimates";
import type { Placement, Todo } from "../../model/types";
import type { BoardColumn } from "../model/board";
import BoardCard from "./BoardCard.svelte";
import { BOARD_DROP_GROUP, type DraggedTodo } from "./dragging";

type BoardColumnProps = {
  column: BoardColumn;
  /** Its todos shown, in the day's order. */
  todos: readonly Todo[];
  tags: readonly Tag[];
  /** Whether todos can be dropped next to its cards, to reorder them: only when all of them are shown. */
  reorderable: boolean;
  /** Called with each todo dropped on it: next to one of its cards (`next`) if dropped on it, else anywhere. */
  onDrop: (
    droppedId: Todo["id"],
    next?: { targetId: Todo["id"]; placement: Placement },
  ) => void;
};
const { column, todos, tags, reorderable, onDrop }: BoardColumnProps = $props();

/** Its title, and what it says while empty. */
const COLUMNS: Record<BoardColumn, { title: string; hint: string }> = {
  todo: { title: "To-Do", hint: "Nothing left to start." },
  urgent: { title: "Urgent", hint: "Drop what can’t wait." },
  doing: { title: "Doing", hint: "Drop what you’re on." },
  done: { title: "Done", hint: "Drop or check off what’s finished." },
};
const { title, hint } = $derived(COLUMNS[column]);
const estimate = $derived(formatEstimates(todos.map((todo) => todo.estimate)));

const headingId = $props.id();
</script>

<!-- Takes the todos dropped anywhere but on its cards: they then go after its last one -->
<section
  class={["column", column]}
  aria-labelledby={headingId}
  {@attach attachDroppable<DraggedTodo>(() => ({
    container: `column:${column}`,
    containerGroup: BOARD_DROP_GROUP,
    attributes: { dragOverClass: "drop-target" },
    callbacks: { onDrop: ({ draggedItem }) => onDrop(draggedItem.id) },
  }))}
>
  <header>
    <span class="dot" aria-hidden="true"></span>
    <h2 id={headingId}>{title}</h2>
    <span class="count" title={`${todos.length} todos`}>{todos.length}</span>
    {#if estimate}
      <span class="estimate" title="Estimated">{estimate}</span>
    {/if}
  </header>
  {#if todos.length > 0}
    <ul class="cards" aria-labelledby={headingId}>
      {#each todos as item (item.id)}
        <BoardCard
          {item}
          {tags}
          {column}
          onDrop={reorderable
            ? (id, placement) => onDrop(id, { targetId: item.id, placement })
            : undefined}
        />
      {/each}
    </ul>
  {:else}
    <p class="hint">{hint}</p>
  {/if}
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .column {
    /* As tall as the board, so that todos can be dropped below its cards */
    @apply flex min-w-0 flex-col gap-2 rounded-xl p-2.5;
    @apply border border-white/5 bg-white/2.5 transition-colors;

    /* Where a dragged todo would go, rather than the library's line: it lands after the column's last card */
    &:global(.drop-target) {
      @apply border-pink-400/50 bg-pink-400/5;
    }
    &:global(.drop-before)::before,
    &:global(.drop-after)::after {
      @apply hidden;
    }
  }

  header {
    @apply flex items-center gap-2 px-1;
  }

  .dot {
    @apply size-2 shrink-0 rounded-full;
    .todo & {
      @apply bg-slate-400;
    }
    .urgent & {
      @apply bg-rose-400;
    }
    .doing & {
      @apply bg-sky-400;
    }
    .done & {
      @apply bg-emerald-400;
    }
  }

  h2 {
    @apply text-xs/6 font-semibold tracking-wider text-slate-200 uppercase;
  }

  .count {
    @apply grid h-5 min-w-5 place-items-center rounded-md bg-white/7 px-1.5;
    @apply text-xs font-semibold text-slate-300 tabular-nums;
  }

  .estimate {
    @apply ml-auto text-xs font-medium text-slate-400 tabular-nums;
  }

  .cards {
    @apply flex flex-col gap-1.5;
  }

  .hint {
    @apply rounded-lg border border-dashed border-white/15 px-3 py-5 text-center text-sm text-slate-400;
    .column:global(.drop-target) & {
      @apply border-pink-400/60 text-pink-300;
    }
  }
</style>
