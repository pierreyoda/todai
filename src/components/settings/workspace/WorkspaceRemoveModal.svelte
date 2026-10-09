<script lang="ts">
  import type { Workspace } from "../../../client/app";
  import Button from "../../common/Button.svelte";
  import FieldText from "../../common/FieldText.svelte";
  import Modal from "../../common/Modal.svelte";

  /** To be typed by the user, to allow the deletion. */
  const CONFIRMATION = "Delete";

  type WorkspaceRemoveModalProps = {
    /** Bindable. */
    show: boolean;
    workspace: Workspace;
    onDelete: () => void;
  };
  let {
    show = $bindable(),
    workspace,
    onDelete,
  }: WorkspaceRemoveModalProps = $props();

  let typed = $state("");
  const confirmed = $derived(typed.trim() === CONFIRMATION);
  $effect(() => {
    if (!show) typed = "";
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
        onDelete();
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
</style>
