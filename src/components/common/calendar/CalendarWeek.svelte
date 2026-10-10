<script lang="ts">
  import type { Day, Tag, Todo } from "../../../client/types";
  import { formatDay, formatWeek, type Week } from "../../../utils/dates";
  import Collapse from "../Collapse.svelte";
  import TodoSummaryItem from "../../todos/TodoSummaryItem.svelte";
  import { formatTodoCounts } from "./counts";

  type CalendarWeekProps = {
    week: Week;
    /** The todos of `week`, by day then in display order. */
    todos: readonly Todo[];
    tags: readonly Tag[];
    selectedTagId: Tag["id"] | null;
    today: Day;
    /** Initial state. */
    open?: boolean;
  };
  const {
    week,
    todos,
    tags,
    selectedTagId,
    today,
    open: initiallyOpen = false,
  }: CalendarWeekProps = $props();
  // svelte-ignore state_referenced_locally: `open` is only the initial state
  let open = $state(initiallyOpen);

  /** Most recent day first; todos keep their display order. */
  const days = $derived.by(() => {
    const todosByDay = new Map<Day, Todo[]>();
    for (const todo of todos) {
      todosByDay.set(todo.day, [...(todosByDay.get(todo.day) ?? []), todo]);
    }
    return [...todosByDay].reverse();
  });
  const completedCount = $derived(
    todos.filter((todo) => todo.completed).length,
  );

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
        {formatTodoCounts(todos.length, completedCount)}
      </span>
    </span>
  {/snippet}
  <div class="days">
    {#each days as [day, dayTodos] (day)}
      <section>
        <h4 class="day">{formatDay(day)}</h4>
        <ul>
          {#each dayTodos as item (item.id)}
            <TodoSummaryItem {item} {tags} {selectedTagId} {today} />
          {/each}
        </ul>
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

  .day {
    @apply px-2 text-xs font-medium text-slate-400;
  }
</style>
