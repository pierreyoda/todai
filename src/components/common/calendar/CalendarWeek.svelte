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
    /** Initial state. */
    open?: boolean;
  };
  const { week, todos, tags, open: initiallyOpen = false }: CalendarWeekProps = $props();
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
  const completedCount = $derived(todos.filter((todo) => todo.completed).length);
</script>

<Collapse bind:open headingLevel={3}>
  {#snippet summary()}
    <span class="summary">
      <span>{formatWeek(week)}</span>
      <span class="counts">{formatTodoCounts(todos.length, completedCount)}</span>
    </span>
  {/snippet}
  <div class="days">
    {#each days as [day, dayTodos] (day)}
      <section>
        <h4 class="day">{formatDay(day)}</h4>
        <ul>
          {#each dayTodos as item (item.id)}
            <TodoSummaryItem {item} {tags} />
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
    @apply shrink-0 text-xs font-medium text-slate-400 tabular-nums;
  }

  .days {
    @apply flex flex-col gap-2 pt-1 pb-2 pl-6;
  }

  .day {
    @apply px-2 text-xs font-medium text-slate-400;
  }
</style>
