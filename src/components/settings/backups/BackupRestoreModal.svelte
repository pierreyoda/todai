<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import {
    invokeApiClient,
    type Workspace,
    type WorkspaceBackup,
  } from "../../../client/app";
  import { invalidateActiveWorkspace } from "../../../client/queries";
  import { formatDateTime } from "../../../utils";
  import Button from "../../common/Button.svelte";
  import ErrorBanner from "../../common/ErrorBanner.svelte";
  import IconArrowUturnLeft from "../../common/icons/IconArrowUturnLeft.svelte";
  import IconShieldCheck from "../../common/icons/IconShieldCheck.svelte";
  import Modal from "../../common/Modal.svelte";

  /** Confirms restoring a backup into its workspace's database. */
  type BackupRestoreModalProps = {
    /** Bindable. */
    show: boolean;
    workspace: Workspace;
    /** One of `workspace`'s. */
    backup: WorkspaceBackup;
  };
  let {
    show = $bindable(),
    workspace,
    backup,
  }: BackupRestoreModalProps = $props();

  const queryClient = useQueryClient();
  // Also refreshes the todos and tags, in case it's the active workspace's
  const restore = createMutation(() => ({
    mutationFn: ({ id }: WorkspaceBackup) =>
      invokeApiClient({ name: "restore_workspace_backup", args: { id } }),
    onSuccess: () => {
      show = false;
      return invalidateActiveWorkspace(queryClient);
    },
  }));
  // A failure is only shown until the modal closes. Untracked, so that resetting doesn't rerun the effect.
  $effect(() => {
    if (!show) untrack(() => restore.reset());
  });
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      Restore {workspace.name}?
    {/snippet}
    <div class="content">
      {#if restore.isError}
        <ErrorBanner
          title="Could not restore the backup"
          error={restore.error}
          onDismiss={() => restore.reset()}
        />
      {/if}
      {#if workspace.available}
        <p>
          Its todos and tags will be replaced by those of its backup from
          <strong>{formatDateTime(backup.createdAt)}</strong>.
        </p>
        <div class="callout">
          <span class="callout-icon">
            <IconShieldCheck class="size-5" />
          </span>
          <p>
            Its current state is backed up first, as <strong>Before restore</strong>:
            restoring that backup undoes this.
          </p>
        </div>
      {:else}
        <p>
          Its database will be recreated at
          <code class="path">{workspace.path}</code> from its backup of
          <strong>{formatDateTime(backup.createdAt)}</strong>.
        </p>
        <p class="hint">
          If a file is still there, nothing is changed: move it away first.
        </p>
      {/if}
    </div>
    {#snippet actions()}
      <Button style="outline" onclick={() => (show = false)}>Cancel</Button>
      <Button disabled={restore.isPending} onclick={() => restore.mutate(backup)}>
        <span data-slot="icon"><IconArrowUturnLeft class="size-full" /></span>
        Restore
      </Button>
    {/snippet}
  </Modal>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .content {
    @apply flex flex-col gap-3;
  }

  strong {
    @apply font-semibold text-white;
  }

  /* Paths can be long: wrapped anywhere rather than overflowing */
  .path {
    @apply rounded bg-white/5 px-1 py-0.5 font-mono text-xs wrap-anywhere text-slate-200;
  }

  .hint {
    @apply text-xs text-slate-400;
  }

  /* Surface: barely lifted from the modal, like the add and import areas from the page */
  .callout {
    @apply flex items-start gap-3 rounded-xl bg-white/3 p-3 ring-1 ring-white/10;
    p {
      @apply pt-1;
    }
  }

  .callout-icon {
    @apply shrink-0 rounded-full bg-blue-500/10 p-1.5 text-blue-400;
  }
</style>
