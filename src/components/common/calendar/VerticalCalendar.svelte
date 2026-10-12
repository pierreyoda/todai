<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import {
    noteMonthsQueryOptions,
    todoMonthsQueryOptions,
  } from "../../../client/queries";
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
   * History of the todos and notes, month by month, most recent first: from the current month back to the one of the
   * earliest todo or note. The current and previous months are initially expanded (unless they have neither), which
   * loads their todos and notes; the others load theirs when first expanded.
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
  const noteMonths = createQuery(() => noteMonthsQueryOptions);

  const statsByMonth = $derived(
    new Map((todoMonths.data ?? []).map((stats) => [stats.month, stats])),
  );
  const monthsWithNotes = $derived(new Set(noteMonths.data ?? []));
  /** Future months are left out; the current one is always listed, even without todos or notes. */
  const months = $derived.by(() => {
    const earliest = [todoMonths.data?.at(-1)?.month, noteMonths.data?.at(-1)]
      .filter((month) => month !== undefined)
      .sort()[0];
    const oldest = earliest && earliest < currentMonth ? earliest : currentMonth;
    return monthsBetween(currentMonth, oldest);
  });
</script>

{#if todoMonths.isPending || noteMonths.isPending}
  <p class="status">Loading…</p>
{:else if todoMonths.isError || noteMonths.isError}
  <p class="status error" role="alert">
    {String(todoMonths.error ?? noteMonths.error)}
  </p>
{:else}
  <div class="calendar">
    {#each months as month (month)}
      {@const stats = statsByMonth.get(month)}
      {@const hasContent = !!stats || monthsWithNotes.has(month)}
      <Collapse
        open={hasContent && (month === currentMonth || month === previousMonth)}
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
        {#if hasContent}
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
