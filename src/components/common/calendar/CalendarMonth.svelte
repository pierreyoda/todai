<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { monthTodosQueryOptions } from "../../../client/queries";
  import type { Day, Month, Tag } from "../../../client/types";
  import { weekOf, weeksOfMonth } from "../../../utils/dates";
  import CalendarWeek from "./CalendarWeek.svelte";

  /** Content of a month: loads its todos when created, i.e. when the month is first expanded. */
  type CalendarMonthProps = {
    month: Month;
    tags: readonly Tag[];
    selectedTagId: Tag["id"] | null;
    today: Day;
  };
  const { month, tags, selectedTagId, today }: CalendarMonthProps = $props();

  const todos = createQuery(() => monthTodosQueryOptions(month));
  const currentWeekStart = $derived(weekOf(today).start);

  /** Most recent first, without the empty ones. */
  const weeks = $derived(
    weeksOfMonth(month)
      .map((week) => ({
        week,
        todos: (todos.data ?? []).filter(
          (todo) => todo.day >= week.start && todo.day <= week.end,
        ),
      }))
      .filter((week) => week.todos.length > 0)
      .reverse(),
  );
</script>

{#if todos.isPending}
  <p class="status">Loading…</p>
{:else if todos.isError}
  <p class="status error" role="alert">{String(todos.error)}</p>
{:else if weeks.length === 0}
  <p class="status">No todos</p>
{:else}
  <div class="weeks">
    {#each weeks as { week, todos } (week.start)}
      <CalendarWeek
        {week}
        {todos}
        {tags}
        {selectedTagId}
        open={week.start === currentWeekStart}
      />
    {/each}
  </div>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .weeks {
    @apply flex flex-col pl-4;
  }

  .status {
    @apply px-2 py-1.5 pl-8 text-sm text-slate-400;
  }

  .error {
    @apply text-red-400;
  }
</style>
