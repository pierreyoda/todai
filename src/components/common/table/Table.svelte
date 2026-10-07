<script lang="ts" generics="Row extends object">
  import type { Snippet } from "svelte";

  type TableColumn = {
    key: keyof Row;
    label: string;
    cell?: Snippet<[row: Row]>;
  };

  type TableProps = {
    columns: readonly TableColumn[];
    rows: readonly Row[];
    /** The property identifying a row, used as the `#each` key. */
    rowKey: keyof Row;
  };
  const { columns, rows, rowKey }: TableProps = $props();
</script>

<table>
  <thead>
    <tr>
      {#each columns as column (column.key)}
        <th>{column.label}</th>
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each rows as row (row[rowKey])}
      <tr>
        {#each columns as column (column.key)}
          <td>
            {#if column.cell}
              {@render column.cell(row)}
            {:else}
              {row[column.key]}
            {/if}
          </td>
        {/each}
      </tr>
    {/each}
  </tbody>
</table>

<style lang="postcss">
</style>
