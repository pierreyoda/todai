<script lang="ts">
  import Button from "../common/Button.svelte";
  import Modal from "../common/Modal.svelte";

  type NoteLeaveModalProps = {
    /** Bindable. Closing it otherwise than by leaving means staying. */
    show: boolean;
    /** Why the note couldn't be saved. */
    error: unknown;
    onLeave: () => void;
  };
  let { show = $bindable(), error, onLeave }: NoteLeaveModalProps = $props();

  const message = $derived(
    typeof error === "string" ? error : error instanceof Error ? error.message : undefined,
  );
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      Today’s note isn’t saved
    {/snippet}
    <div class="content">
      <p>
        Its latest changes couldn’t be saved: leaving now loses them. Stay to try again, or to copy them elsewhere
        first.
      </p>
      {#if message}
        <p class="error">{message}</p>
      {/if}
    </div>
    {#snippet actions()}
      <Button style="outline" onclick={() => (show = false)}>Stay</Button>
      <Button color="red" onclick={onLeave}>Leave anyway</Button>
    {/snippet}
  </Modal>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .content {
    @apply flex flex-col gap-3;
  }

  /* Messages hold paths: wrapped anywhere rather than overflowing */
  .error {
    @apply rounded-lg bg-red-500/10 px-3 py-2 text-xs/5 wrap-anywhere text-red-200/80 ring-1 ring-red-500/20;
  }
</style>
