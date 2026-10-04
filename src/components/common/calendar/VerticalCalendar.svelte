<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { todoMonthsQueryOptions } from "../../../client/queries";
  import type { Day, Tag } from "../../../client/types";
  import {
    addMonths,
    dateToTodaiDate,
    formatMonth,
    monthOf,
    monthsBetween,
  } from "../../../utils/dates";
  import Collapse from "../Collapse.svelte";
  import CalendarMonth from "./CalendarMonth.svelte";
  import { formatTodoCounts } from "./counts";

  /**
   * History of the todos, month by month, most recent first: from the current month back to the one of the earliest
   * todo. The current and previous months are initially expanded (unless they have no todos), which loads their todos;
   * the others load theirs when first expanded.
   */
  type VerticalCalendarProps = {
    tags: readonly Tag[];
    selectedTagId: Tag["id"] | null;
    /** Fixed once mounted, like the day of the todos panel. */
    today?: Day;
  };
  const {
    tags,
    selectedTagId,
    today = dateToTodaiDate(new Date()),
  }: VerticalCalendarProps = $props();

  // svelte-ignore state_referenced_locally: fixed once mounted
  const currentMonth = monthOf(today);
  const previousMonth = addMonths(currentMonth, -1);

  const todoMonths = createQuery(() => todoMonthsQueryOptions);

  const statsByMonth = $derived(
    new Map((todoMonths.data ?? []).map((stats) => [stats.month, stats])),
  );
  /** Future months are left out; the current and previous ones are always listed, even without todos. */
  const months = $derived.by(() => {
    const earliest = todoMonths.data?.at(-1)?.month;
    const oldest =
      earliest && earliest < previousMonth ? earliest : previousMonth;
    return monthsBetween(currentMonth, oldest);
  });
</script>

{#if todoMonths.isPending}
  <p class="status">Loading…</p>
{:else if todoMonths.isError}
  <p class="status error" role="alert">{String(todoMonths.error)}</p>
{:else}
  <div class="calendar">
    {#each months as month (month)}
      {@const stats = statsByMonth.get(month)}
      <Collapse
        open={!!stats && (month === currentMonth || month === previousMonth)}
        headingLevel={2}
      >
        {#snippet summary()}
          <span class="summary">
            <span>{formatMonth(month)}</span>
            <span class="counts">
              {stats
                ? formatTodoCounts(stats.count, stats.completedCount)
                : "No todos"}
            </span>
          </span>
        {/snippet}
        {#if stats}
          <CalendarMonth {month} {tags} {selectedTagId} {today} />
        {:else}
          <!-- Nothing to load -->
          <p class="status">No todos</p>
        {/if}
      </Collapse>
    {/each}
  </div>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .calendar {
    @apply w-full flex flex-col gap-1;
  }

  .summary {
    @apply flex items-baseline justify-between gap-2;
  }

  .counts {
    @apply shrink-0 text-xs font-medium text-slate-400 tabular-nums;
  }

  .status {
    @apply px-2 py-1.5 pl-8 text-sm text-slate-400;
  }

  .error {
    @apply text-red-400;
  }
</style>
