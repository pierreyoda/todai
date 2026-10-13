<script lang="ts">
import type { Day } from "#core/time";
import { formatDay, formatFullDay, formatWeek, type Week } from "#core/time";
import type { Note } from "#domains/notes";
import { NoteMarker, notePath } from "#domains/notes";
import type { Tag } from "#domains/tags";
import type { Todo } from "#domains/todos";
import { TodoSummaryItem } from "#domains/todos";
import Collapse from "#shared/ui/Collapse.svelte";
import IconDocumentText from "#shared/ui/icons/IconDocumentText.svelte";
import { formatTodoCounts } from "../model/counts";

type CalendarWeekProps = {
  week: Week;
  /** The todos of `week`, by day then in display order. */
  todos: readonly Todo[];
  /** The notes of `week`: their days are listed too, even without todos. */
  notes?: readonly Note[];
  tags: readonly Tag[];
  selectedTagId: Tag["id"] | null;
  today: Day;
  /** Initial state. */
  open?: boolean;
};
const {
  week,
  todos,
  notes = [],
  tags,
  selectedTagId,
  today,
  open: initiallyOpen = false,
}: CalendarWeekProps = $props();
// svelte-ignore state_referenced_locally: `open` is only the initial state
let open = $state(initiallyOpen);

/** Most recent day first, with its todos in display order and its note, if any. */
const days = $derived.by(() => {
  const byDay = new Map<Day, { todos: Todo[]; note?: Note }>();
  const dayOf = (day: Day) => {
    let entry = byDay.get(day);
    if (!entry) byDay.set(day, (entry = { todos: [] }));
    return entry;
  };
  for (const todo of todos) dayOf(todo.day).todos.push(todo);
  for (const note of notes) dayOf(note.day).note = note;
  return [...byDay].sort(([a], [b]) => b.localeCompare(a));
});
const completedCount = $derived(todos.filter((todo) => todo.completed).length);

/** Shows which weeks have todos with the selected tag, even collapsed. */
const selectedTag = $derived(tags.find((tag) => tag.id === selectedTagId));
const selectedTagCount = $derived(
  selectedTag
    ? todos.filter((todo) => todo.tagIds.includes(selectedTag.id)).length
    : 0,
);
</script>

<Collapse bind:open headingLevel={3}>
  {#snippet summary()}
    <span class="summary">
      <span>{formatWeek(week)}</span>
      <span class="counts">
        {#if selectedTag && selectedTagCount > 0}
          <span class="tagged" style:--tag-color={selectedTag.color}>
            <span class="tagged-dot" aria-hidden="true"></span>
            {selectedTagCount}<span class="sr-only">
              tagged {selectedTag.name}</span
            >
          </span>
        {/if}
        {todos.length > 0 ? formatTodoCounts(todos.length, completedCount) : "No todos"}
      </span>
    </span>
  {/snippet}
  <div class="days">
    {#each days as [day, { todos: dayTodos, note }] (day)}
      <section class="day-section">
        <div class="day-header">
          <h4 class="day">{formatDay(day)}</h4>
          {#if note}
            <NoteMarker {note} {today} />
          {:else}
            <!-- Where its note's marker would be, grayed: shown with the day's hover or focus -->
            <a
              class="add-note"
              href={notePath(day, today)}
              title="Add a note"
              aria-label={`Add a note for ${formatFullDay(day)}`}
            >
              <IconDocumentText />
            </a>
          {/if}
        </div>
        {#if dayTodos.length > 0}
          <ul>
            {#each dayTodos as item (item.id)}
              <TodoSummaryItem {item} {tags} {selectedTagId} {today} />
            {/each}
          </ul>
        {:else}
          <!-- Listed for its note -->
          <p class="no-todos">No todos</p>
        {/if}
      </section>
    {/each}
  </div>
</Collapse>

<style lang="postcss">
  @reference "tailwindcss";

  .summary {
    @apply flex items-baseline justify-between gap-2;
  }

  .counts {
    @apply flex shrink-0 items-center gap-2 text-xs font-medium text-slate-400 tabular-nums;
  }

  .tagged {
    @apply flex items-center gap-1 text-white;
    .tagged-dot {
      @apply size-1.5 rounded-full bg-(--tag-color);
    }
  }

  .days {
    @apply flex flex-col gap-2 pt-1 pb-2 pl-6;
  }

  /* The day, and its note's marker right after it: as tall as the marker, whether the day has a note or not */
  .day-header {
    @apply flex h-5 items-center gap-1 px-2;
  }

  .day {
    @apply text-xs font-medium text-slate-400;
  }

  /* As the note's marker, in its box, grayed until hovered */
  .add-note {
    @apply flex h-5.5 w-6 shrink-0 items-center justify-center rounded-md text-slate-500 opacity-0 transition;
    @apply hover:bg-white/10 hover:text-slate-300;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
    > :global(svg) {
      @apply size-3.5;
    }
  }
  .day-section:hover .add-note,
  .day-section:focus-within .add-note {
    @apply opacity-100;
  }

  /* As a todo's row */
  .no-todos {
    @apply px-2 py-1 text-sm text-slate-500;
  }
</style>
