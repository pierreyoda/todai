<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import {
    invokeApiClient,
    type Workspace,
    type WorkspaceBackup,
  } from "../../../client/app";
  import { invalidateActiveWorkspace } from "../../../client/queries";
  import { formatDateTime, formatShortDate } from "../../../utils";
  import ErrorBanner from "../../common/ErrorBanner.svelte";
  import Modal from "../../common/Modal.svelte";
  import WorkspaceForm, {
    type WorkspaceFormData,
  } from "../workspace/WorkspaceForm.svelte";

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

  const queryClient = useQueryClient();
  // Restoring as a new workspace switches to it.
  const restoreAsNew = createMutation(() => ({
    mutationFn: ({ name, path }: WorkspaceFormData) =>
      invokeApiClient({
        name: "restore_workspace_backup_as_new",
        args: { id: backup.id, name, path },
      }),
    onSuccess: (restored) => {
      show = false;
      onRestored?.(restored);
      return invalidateActiveWorkspace(queryClient);
    },
  }));
  // A failure is only shown until the modal closes. Untracked, so that resetting doesn't rerun the effect.
  $effect(() => {
    if (!show) untrack(() => restoreAsNew.reset());
  });
</script>

<div class="modal-container">
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
        onSubmit={(data) => restoreAsNew.mutate(data)}
      />
    </div>
  </Modal>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .content {
    @apply flex flex-col gap-4;
  }

  strong {
    @apply font-semibold text-white;
  }
</style>
