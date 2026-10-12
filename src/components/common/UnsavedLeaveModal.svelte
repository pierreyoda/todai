<script lang="ts">
  import Button from "./Button.svelte";
  import Modal from "./Modal.svelte";

  type UnsavedLeaveModalProps = {
    /** Bindable. Closing it otherwise than by leaving means staying. */
    show: boolean;
    /** What isn't saved, starting a sentence: e.g. "Today’s note", or "The note of Saturday, October 10, 2026". */
    name: string;
    /** How it's left: by going elsewhere (e.g. another page), or by closing what it's edited in (e.g. a modal). */
    leaving?: "leave" | "close";
    /** Why it couldn't be saved. */
    error: unknown;
    onLeave: () => void;
  };
  let { show = $bindable(), name, leaving = "leave", error, onLeave }: UnsavedLeaveModalProps = $props();

  const WORDING = {
    leave: { now: "leaving now", button: "Leave anyway" },
    close: { now: "closing now", button: "Close anyway" },
  };
  const wording = $derived(WORDING[leaving]);

  const message = $derived(
    typeof error === "string" ? error : error instanceof Error ? error.message : undefined,
  );
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      {name} isn’t saved
    {/snippet}
    <div class="content">
      <p>
        Its latest changes couldn’t be saved: {wording.now} loses them. Stay to try again, or to copy them elsewhere
        first.
      </p>
      {#if message}
        <p class="error">{message}</p>
      {/if}
    </div>
    {#snippet actions()}
      <Button style="outline" onclick={() => (show = false)}>Stay</Button>
      <Button color="red" onclick={onLeave}>{wording.button}</Button>
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
