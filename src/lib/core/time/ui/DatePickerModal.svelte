<script lang="ts">
import { tick, untrack } from "svelte";
import Button from "#shared/ui/Button.svelte";
import Modal from "#shared/ui/Modal.svelte";
import type { Day } from "../days";
import {
  addDays,
  addMonths,
  calendarWeeksOfMonth,
  dateToTodaiDate,
  formatFullDay,
  formatMonth,
  monthBounds,
  monthOf,
} from "../days";

/**
 * A month calendar in a modal, to pick a day: see `DatePicker` for a button opening it.
 *
 * The calendar follows the WAI-ARIA date picker dialog pattern: arrows move by day and week, Home and End to the
 * week's bounds, Page Up and Page Down by month (by year with Shift), Enter or Space picks the focused day.
 */
type DatePickerModalProps = {
  /** Bindable: whether it's shown. Closed once a day is picked. */
  show: boolean;
  /** The selected day, if any: focused when opened, rather than today. */
  day?: Day | null;
  /** Title of the modal. */
  label?: string;
  onPick: (day: Day) => void;
};
let {
  show = $bindable(),
  day = null,
  label,
  onPick,
}: DatePickerModalProps = $props();

const WEEKDAYS = [
  ["Mo", "Monday"],
  ["Tu", "Tuesday"],
  ["We", "Wednesday"],
  ["Th", "Thursday"],
  ["Fr", "Friday"],
  ["Sa", "Saturday"],
  ["Su", "Sunday"],
] as const;

const monthId = $props.id();

/** Updated when opening, so that a modal opened after midnight shows the right day. */
let today = $state(dateToTodaiDate(new Date()));
/** The day focusable in the calendar (roving tabindex), which also sets the displayed month. */
let focusedDay = $state<Day>(dateToTodaiDate(new Date()));
let grid = $state<HTMLTableElement>();

const month = $derived(monthOf(focusedDay));
const weeks = $derived(calendarWeeksOfMonth(month));

/** Whether the month `count` months away can be displayed: its weeks must stay within 0000-9999. */
const canShowMonth = (count: number) => {
  try {
    calendarWeeksOfMonth(addMonths(month, count));
    return true;
  } catch {
    return false;
  }
};

/** The same day of the month `count` months away, or its last day if shorter (e.g. Jan 31 => Feb 28). */
const shiftMonths = (from: Day, count: number): Day => {
  const [, last] = monthBounds(addMonths(monthOf(from), count));
  const sameDay = `${last.slice(0, 8)}${from.slice(8)}`;
  return sameDay <= last ? sameDay : last;
};

const focusDayButton = () =>
  grid?.querySelector<HTMLButtonElement>(`[data-day="${focusedDay}"]`)?.focus();

/**
 * Makes `getTarget()` the focused day, ignored when it can't be displayed (out of 0000-9999).
 *
 * @param moveFocus Whether to also move the browser's focus to it: not for the month buttons, which keep it.
 */
const setFocusedDay = async (getTarget: () => Day, moveFocus = true) => {
  let target: Day;
  try {
    target = getTarget();
    calendarWeeksOfMonth(monthOf(target));
  } catch {
    return;
  }
  focusedDay = target;
  if (!moveFocus) return;
  // The target may be in another month, rendered on the next update
  await tick();
  focusDayButton();
};

// Each time it's opened
$effect(() => {
  if (!show) return;
  untrack(() => {
    today = dateToTodaiDate(new Date());
    focusedDay = day ?? today;
  });
  // Once the modal's `showModal()` ran, which focuses its close button: the focused day takes over
  void tick().then(focusDayButton);
});

const pick = (pickedDay: Day) => {
  show = false;
  onPick(pickedDay);
};

const onkeydown = (event: KeyboardEvent) => {
  const week = weeks.find((days) => days.includes(focusedDay));
  const years = event.shiftKey ? 12 : 1;
  const moves: Partial<Record<string, () => Day>> = {
    ArrowLeft: () => addDays(focusedDay, -1),
    ArrowRight: () => addDays(focusedDay, 1),
    ArrowUp: () => addDays(focusedDay, -7),
    ArrowDown: () => addDays(focusedDay, 7),
    Home: () => week![0],
    End: () => week![6],
    PageUp: () => shiftMonths(focusedDay, -years),
    PageDown: () => shiftMonths(focusedDay, years),
  };
  const move = moves[event.key];
  if (!move || !week) return;
  event.preventDefault();
  setFocusedDay(move);
};
</script>

<Modal bind:show>
  {#snippet header()}
    {label}
  {/snippet}
  <div class="calendar">
    <div class="month-navigation">
      <Button
        style="plain"
        size="sm"
        aria-label="Previous month"
        disabled={!canShowMonth(-1)}
        onclick={() =>
          setFocusedDay(() => shiftMonths(focusedDay, -1), false)}
      >
        <svg
          data-slot="icon"
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fill-rule="evenodd"
            d="M11.78 5.22a.75.75 0 0 1 0 1.06L8.06 10l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"
            clip-rule="evenodd"
          />
        </svg>
      </Button>
      <h3 id={monthId} class="month" aria-live="polite">
        {formatMonth(month)}
      </h3>
      <Button
        style="plain"
        size="sm"
        aria-label="Next month"
        disabled={!canShowMonth(1)}
        onclick={() => setFocusedDay(() => shiftMonths(focusedDay, 1), false)}
      >
        <svg
          data-slot="icon"
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fill-rule="evenodd"
            d="M8.22 5.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L11.94 10 8.22 6.28a.75.75 0 0 1 0-1.06Z"
            clip-rule="evenodd"
          />
        </svg>
      </Button>
    </div>
    <table bind:this={grid} role="grid" aria-labelledby={monthId} {onkeydown}>
      <thead>
        <tr>
          {#each WEEKDAYS as [short, long] (long)}
            <th scope="col" abbr={long}>{short}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each weeks as week (week[0])}
          <tr>
            {#each week as weekDay (weekDay)}
              <td role="gridcell" aria-selected={weekDay === day}>
                <button
                  type="button"
                  data-day={weekDay}
                  tabindex={weekDay === focusedDay ? 0 : -1}
                  aria-label={formatFullDay(weekDay)}
                  aria-current={weekDay === today ? "date" : undefined}
                  class={[
                    "day",
                    monthOf(weekDay) !== month && "outside",
                    weekDay === today && "today",
                    weekDay === day && "selected",
                  ]}
                  onclick={() => pick(weekDay)}
                >
                  {Number(weekDay.slice(8))}
                </button>
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  {#snippet actions()}
    <Button style="outline" onclick={() => pick(today)}>Today</Button>
  {/snippet}
</Modal>

<style lang="postcss">
  @reference "tailwindcss";

  .calendar {
    @apply flex w-full p-2 flex-col gap-2;
  }

  .month-navigation {
    @apply flex items-center justify-between gap-2 p-2;
  }

  .month {
    @apply text-sm font-semibold text-white;
  }

  table {
    @apply w-full border-collapse text-center;
  }

  th {
    @apply pb-1 text-xs font-medium text-slate-500;
  }

  td {
    @apply p-0.5;
  }

  .day {
    /* Layout */
    @apply mx-auto flex size-9 items-center justify-center rounded-lg;
    /* Typography */
    @apply text-sm text-slate-200 tabular-nums;
    /* States */
    @apply cursor-pointer transition hover:bg-white/10;
    @apply focus:outline-hidden focus-visible:ring-2 focus-visible:ring-blue-500;
    /* The adjacent months' days */
    &.outside {
      @apply text-slate-500;
    }
    &.today {
      @apply font-semibold text-fuchsia-300 ring-1 ring-white/15 ring-inset;
    }
    &.selected {
      @apply bg-fuchsia-500 font-semibold text-white hover:bg-fuchsia-400;
      /* Forced colors mode: backgrounds are dropped */
      @apply forced-colors:outline;
    }
  }
</style>
