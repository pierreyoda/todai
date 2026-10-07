<script lang="ts">
  import type { Workspace } from "../../../client/app";
  import { basename } from "../../../utils";
  import Table from "../../common/table/Table.svelte";

  const workspaces: readonly Workspace[] = [];
</script>

{#snippet pathCell({ path }: Workspace)}
  <span title={path}>{basename(path)}</span>
{/snippet}

{#snippet lastOpenedAtCell({ lastOpenedAt }: Workspace)}
  <span>{lastOpenedAt}</span>
{/snippet}

<Table
  rows={workspaces}
  columns={[
    {
      key: "name",
      label: "Name",
    },
    {
      key: "path",
      label: "Path",
      cell: pathCell,
    },
    {
      key: "lastOpenedAt",
      label: "Last opened",
      cell: lastOpenedAtCell,
    },
  ]}
  rowKey="id"
/>
