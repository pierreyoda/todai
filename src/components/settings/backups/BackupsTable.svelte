<script lang="ts">
  import { revealItemInDir } from "@tauri-apps/plugin-opener";

  import type { WorkspaceBackup } from "../../../client/app";
  import {
    basename,
    formatDateTime,
    formatFileSize,
    formatFullDateTime,
    SHOW_IN_FILE_MANAGER,
  } from "../../../utils";
  import Button from "../../common/Button.svelte";
  import ConfirmButton from "../../common/ConfirmButton.svelte";
  import IconArchiveBox from "../../common/icons/IconArchiveBox.svelte";
  import IconArrowUturnLeft from "../../common/icons/IconArrowUturnLeft.svelte";
  import IconDocumentDuplicate from "../../common/icons/IconDocumentDuplicate.svelte";
  import IconFolderOpen from "../../common/icons/IconFolderOpen.svelte";
  import IconTrash from "../../common/icons/IconTrash.svelte";
  import Table from "../../common/table/Table.svelte";
  import BackupKindBadge from "./BackupKindBadge.svelte";

  type BackupsTableProps = {
    /** Most recent first. */
    backups: readonly WorkspaceBackup[];
    /** Of the workspace they're the backups of. */
    workspaceName: string;
    /** Whether that workspace is backed up daily. */
    autoBackup: boolean;
    /** Into its workspace's database. */
    onRestore: (backup: WorkspaceBackup) => void;
    onRestoreAsNew: (backup: WorkspaceBackup) => void;
    onDelete: (backup: WorkspaceBackup) => void;
  };
  const {
    backups,
    workspaceName,
    autoBackup,
    onRestore,
    onRestoreAsNew,
    onDelete,
  }: BackupsTableProps = $props();
</script>

{#snippet createdAtCell({ createdAt, path }: WorkspaceBackup)}
  <time
    datetime={createdAt.toISOString()}
    title={`${formatFullDateTime(createdAt)} · ${basename(path)}`}
  >
    {formatDateTime(createdAt)}
  </time>
{/snippet}

{#snippet kindCell({ kind }: WorkspaceBackup)}
  <BackupKindBadge {kind} />
{/snippet}

{#snippet sizeCell({ available, size }: WorkspaceBackup)}
  {#if available && size !== undefined}
    {formatFileSize(size)}
  {:else}
    <!-- Its file was deleted or moved outside of the app -->
    <span class="missing" title="Its file is missing">Missing</span>
  {/if}
{/snippet}

{#snippet actionsCell(backup: WorkspaceBackup)}
  {@const label = `backup of ${formatDateTime(backup.createdAt)}`}
  <div class="actions">
    <Button
      style="plain"
      size="sm"
      title="Restore"
      aria-label={`Restore the ${label}`}
      disabled={!backup.available}
      onclick={() => onRestore(backup)}
    >
      <IconArrowUturnLeft />
    </Button>
    <Button
      style="plain"
      size="sm"
      title="Restore as a new workspace"
      aria-label={`Restore the ${label} as a new workspace`}
      disabled={!backup.available}
      onclick={() => onRestoreAsNew(backup)}
    >
      <IconDocumentDuplicate />
    </Button>
    <Button
      style="plain"
      size="sm"
      title={SHOW_IN_FILE_MANAGER}
      aria-label={`${SHOW_IN_FILE_MANAGER}: ${label}`}
      disabled={!backup.available}
      onclick={() =>
        revealItemInDir(backup.path).catch((error) =>
          console.error(`Could not reveal ${backup.path}:`, error),
        )}
    >
      <IconFolderOpen />
    </Button>
    <ConfirmButton
      style="plain"
      size="sm"
      title="Delete"
      aria-label={`Delete the ${label}`}
      onclick={() => onDelete(backup)}
    >
      <IconTrash class="text-red-500" />
    </ConfirmButton>
  </div>
{/snippet}

{#snippet empty()}
  <span class="empty-state">
    <span class="empty-icon">
      <IconArchiveBox class="size-6" />
    </span>
    <span class="empty-title">No backups yet</span>
    <span class="empty-hint">
      Backups are copies of {workspaceName}'s todos and tags, kept in the app's
      data folder.
      {#if autoBackup}
        One is made automatically each day it's open.
      {/if}
    </span>
  </span>
{/snippet}

<!-- Fills the page, scrolling past it -->
<div class="frame">
  <Table
    rows={backups}
    columns={[
      {
        key: "createdAt",
        label: "Created",
        cell: createdAtCell,
      },
      {
        key: "kind",
        label: "Type",
        cell: kindCell,
      },
      {
        key: "size",
        label: "Size",
        align: "end",
        cell: sizeCell,
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
    {empty}
  />
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .frame {
    /* Layout */
    @apply min-h-0 w-full flex-1 overflow-y-auto;
    /* Frame */
    @apply rounded-xl ring-1 ring-white/10;
  }

  .missing {
    @apply text-slate-500 italic;
  }

  .actions {
    @apply inline-flex items-center gap-1;
  }

  /* In the table's own empty cell, already padded and centered */
  .empty-state {
    @apply flex flex-col items-center gap-1;
    .empty-icon {
      @apply mb-2 flex size-12 items-center justify-center rounded-full;
      @apply bg-white/5 text-slate-300 ring-1 ring-white/10 ring-inset;
    }
    .empty-title {
      @apply text-sm font-medium text-white;
    }
    .empty-hint {
      @apply max-w-xs text-xs text-slate-400;
    }
  }
</style>
