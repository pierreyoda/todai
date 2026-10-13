<script lang="ts">
  import { untrack } from "svelte";

  import type { Workspace } from "../../model/types";
  import type { WorkspaceBackup } from "../model/types";
  import { createRestoreBackupAsNewMutation } from "../api/mutations";
  import { formatDateTime, formatShortDate } from "#shared/utils/timestamps.ts";
  import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
  import Modal from "#shared/ui/Modal.svelte";
  import WorkspaceForm from "../../ui/WorkspaceForm.svelte";

  /** Creates a workspace from a backup, with a copy of it as its database. Only reads `backup` initially. */
  type BackupRestoreAsNewModalProps = {
    /** Bindable. */
    show: boolean;
    /** The one `backup` is of, left as it is. */
    workspace: Workspace;
    backup: WorkspaceBackup;
    /** With the new workspace, now the active one. */
    onRestored?: (restored: Workspace) => void;
  };
  let {
    show = $bindable(),
    workspace,
    backup,
    onRestored,
  }: BackupRestoreAsNewModalProps = $props();

  // Restoring as a new workspace switches to it
  const restoreAsNew = createRestoreBackupAsNewMutation({
    onSuccess: (restored) => {
      show = false;
      onRestored?.(restored);
    },
  });
  // A failure is only shown until the modal closes. Untracked, so that resetting doesn't rerun the effect.
  $effect(() => {
    if (!show) untrack(() => restoreAsNew.reset());
  });
</script>

<Modal bind:show>
  {#snippet header()}
    Restore as a new workspace
  {/snippet}
  <div class="content">
    {#if restoreAsNew.isError}
      <ErrorBanner
        title="Could not restore the backup"
        error={restoreAsNew.error}
        onDismiss={() => restoreAsNew.reset()}
      />
    {/if}
    <p>
      A copy of {workspace.name}'s backup from
      <strong>{formatDateTime(backup.createdAt)}</strong>, as another
      workspace: {workspace.name} stays as it is, and you then switch to the copy.
    </p>
    <WorkspaceForm
      initialName={`${workspace.name} (${formatShortDate(backup.createdAt)})`}
      submitLabel="Restore"
      onSubmit={(data) => restoreAsNew.mutate({ backup, ...data })}
    />
  </div>
</Modal>

<style lang="postcss">
  @reference "tailwindcss";

  .content {
    @apply flex flex-col gap-4;
  }

  strong {
    @apply font-semibold text-white;
  }
</style>
