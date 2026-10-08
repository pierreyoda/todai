<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import { basename, dateToTodaiDate } from "../../../utils";
  import Button from "../../common/Button.svelte";
  import IconFolderOpen from "../../common/icons/IconFolderOpen.svelte";
  import IconPencilSquare from "../../common/icons/IconPencilSquare.svelte";
  import Table from "../../common/table/Table.svelte";
  import { invokeApiClient, type Workspace } from "../../../client/app";
  import { invalidateActiveWorkspace } from "../../../client/queries";

  type WorkspacesTableProps = {
    workspaces: readonly Workspace[];
    onEdit: (workspace: Workspace) => void;
  };
  const { workspaces, onEdit }: WorkspacesTableProps = $props();

  const queryClient = useQueryClient();
  const switchWorkspace = createMutation(() => ({
    mutationFn: ({ id }: Workspace) =>
      invokeApiClient({ name: "switch_to_workspace", args: { id } }),
    onSuccess: () => invalidateActiveWorkspace(queryClient),
  }));
</script>

{#snippet pathCell({ path }: Workspace)}
  <span class="path" title={path}>{basename(path)}</span>
{/snippet}

{#snippet lastOpenedAtCell({ lastOpenedAt }: Workspace)}
  {#if lastOpenedAt}
    <time
      datetime={lastOpenedAt.toISOString()}
      title={lastOpenedAt.toLocaleString()}
    >
      {dateToTodaiDate(lastOpenedAt)}
    </time>
  {:else}
    <span class="never">Never</span>
  {/if}
{/snippet}

{#snippet actionsCell(workspace: Workspace)}
  <div class="actions">
    <Button
      style="plain"
      size="sm"
      title="Switch to this workspace"
      aria-label={`Switch to ${workspace.name}`}
      disabled={workspace.isActive ||
        !workspace.available ||
        switchWorkspace.isPending}
      onclick={() => switchWorkspace.mutate(workspace)}
    >
      <IconFolderOpen />
    </Button>
    <Button
      style="plain"
      size="sm"
      title="Edit"
      aria-label={`Edit ${workspace.name}`}
      disabled={!workspace.available}
      onclick={() => onEdit(workspace)}
    >
      <IconPencilSquare />
    </Button>
  </div>
{/snippet}

<!-- Fixed height, scrolling past it: the add area above keeps the free space whatever the number of workspaces -->
<div class="sizer">
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
      {
        id: "actions",
        label: "Actions",
        hideLabel: true,
        align: "end",
        cell: actionsCell,
      },
    ]}
    rowKey="id"
  />
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .sizer {
    /* Layout */
    @apply h-64 w-full shrink-0 overflow-y-auto;
    /* Frame */
    @apply rounded-xl ring-1 ring-white/10;
  }

  .path {
    @apply font-mono text-xs text-slate-400;
  }

  .never {
    @apply text-slate-500 italic;
  }

  .actions {
    @apply inline-flex items-center gap-1;
  }
</style>
