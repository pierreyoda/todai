<script lang="ts" generics="Row extends object">
  import type { Snippet } from "svelte";

  type TableColumn = {
    label: string;
    /** Keeps the label for screen readers only. */
    hideLabel?: boolean;
    align?: "start" | "end";
  } & (
    | {
        /** Displays `row[key]`, or `cell` if given. */
        key: keyof Row;
        cell?: Snippet<[row: Row]>;
      }
    | {
        /** Not tied to a property of the row (e.g. actions), so `cell` is required. */
        id: string;
        cell: Snippet<[row: Row]>;
      }
  );

  type TableProps = {
    columns: readonly TableColumn[];
    rows: readonly Row[];
    /** The property identifying a row, used as the `#each` key. */
    rowKey: keyof Row;
  };
  const { columns, rows, rowKey }: TableProps = $props();

  const columnId = (column: TableColumn): string =>
    "key" in column ? String(column.key) : column.id;
</script>

<table>
  <thead>
    <tr>
      {#each columns as column (columnId(column))}
        <th scope="col" class={[column.align === "end" && "end"]}>
          <span class={[column.hideLabel && "sr-only"]}>{column.label}</span>
        </th>
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each rows as row (row[rowKey])}
      <tr>
        {#each columns as column (columnId(column))}
          <td class={[column.align === "end" && "end"]}>
            {#if column.cell}
              {@render column.cell(row)}
            {:else if "key" in column}
              {row[column.key]}
            {/if}
          </td>
        {/each}
      </tr>
    {:else}
      <tr>
        <td class="empty" colspan={columns.length}>Nothing to show.</td>
      </tr>
    {/each}
  </tbody>
</table>

<style lang="postcss">
  @reference "tailwindcss";

  table {
    /* Separate borders, so that the header keeps its own while sticking */
    @apply w-full border-separate border-spacing-0;
    /* Typography */
    @apply text-left text-sm text-slate-300;
  }

  th {
    /* Sticks to the top of a scrolling container, opaque so that rows scroll under it */
    @apply sticky top-0 z-10 bg-slate-900;
    /* Layout */
    @apply border-b border-white/10 px-4 py-3;
    /* Typography */
    @apply text-xs font-semibold tracking-wider whitespace-nowrap text-slate-400 uppercase;
  }

  td {
    /* Layout */
    @apply border-b border-white/5 px-4 py-2.5;
    /* Typography: aligned digits, for dates and amounts */
    @apply tabular-nums;
    /* Animated row hover */
    @apply transition-colors duration-150 motion-reduce:transition-none;

    &:first-child {
      @apply font-medium text-white;
    }

    &.empty {
      @apply py-8 text-center font-normal text-slate-500;
    }
  }

  th,
  td {
    &.end {
      @apply text-right;
    }
  }

  tbody tr {
    &:last-child > td {
      @apply border-b-0;
    }

    &:hover > td:not(.empty) {
      @apply bg-white/5;
    }
  }
</style>
