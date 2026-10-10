<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import type { Workspace } from "../../../client/app";
  import { workspaceBackupsQueryOptions } from "../../../client/queries";
  import { formatFileSize } from "../../../utils";
  import Button from "../../common/Button.svelte";
  import FieldCheckbox from "../../common/FieldCheckbox.svelte";
  import FieldText from "../../common/FieldText.svelte";
  import Modal from "../../common/Modal.svelte";

  /** To be typed by the user, to allow the deletion. */
  const CONFIRMATION = "Delete";

  type WorkspaceRemoveModalProps = {
    /** Bindable. */
    show: boolean;
    workspace: Workspace;
    /** With whether its backups are to be deleted too. */
    onDelete: (deleteBackups: boolean) => void;
  };
  let {
    show = $bindable(),
    workspace,
    onDelete,
  }: WorkspaceRemoveModalProps = $props();

  const backups = createQuery(() => ({
    ...workspaceBackupsQueryOptions(workspace.id),
    enabled: show,
  }));
  const backupsCount = $derived(backups.data?.length ?? 0);
  const backupsSize = $derived(
    (backups.data ?? []).reduce((total, { size }) => total + (size ?? 0), 0),
  );

  let typed = $state("");
  const confirmed = $derived(typed.trim() === CONFIRMATION);
  // Kept by default: they can still be restored as a new workspace, from the files
  let deleteBackups = $state(false);
  $effect(() => {
    if (!show) {
      typed = "";
      deleteBackups = false;
    }
  });

  const formId = $props.id();
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      {workspace.name}
    {/snippet}
    <form
      id={formId}
      class="content"
      onsubmit={(event) => {
        event.preventDefault();
        if (!confirmed) return;
        onDelete(deleteBackups && backupsCount > 0);
      }}
    >
      <p>
        <strong>{workspace.name}</strong> will be removed from your workspaces.
      </p>
      <p>
        Its database is kept at <code class="path">{workspace.path}</code>: you
        can import it again later.
      </p>
      {#if workspace.isActive}
        <p>
          As it's the active workspace, you'll then switch to another one, or
          create one.
        </p>
      {/if}
      {#if backupsCount > 0}
        <div class="backups">
          <FieldCheckbox
            label={`Also delete its ${backupsCount} ${backupsCount === 1 ? "backup" : "backups"} (${formatFileSize(backupsSize)})`}
            description="Otherwise they're kept in the app's data folder, no longer listed."
            bind:checked={deleteBackups}
          />
        </div>
      {/if}
      <FieldText
        bind:value={typed}
        label={`Type "${CONFIRMATION}" to confirm`}
        placeholder={CONFIRMATION}
        autocomplete="off"
        spellcheck={false}
      />
    </form>
    {#snippet actions()}
      <Button style="outline" onclick={() => (show = false)}>Cancel</Button>
      <Button type="submit" form={formId} color="red" disabled={!confirmed}>
        Delete
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

  /* Surface: barely lifted from the modal, setting the choice apart */
  .backups {
    @apply rounded-xl bg-white/3 p-3 ring-1 ring-white/10;
  }
</style>
