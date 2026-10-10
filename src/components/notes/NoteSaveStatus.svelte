<script lang="ts">
  import type { AutosaveStatus } from "../../utils/autosave.svelte";
  import { formatFullDateTime, formatTime } from "../../utils/dates";
  import IconCheckCircle from "../common/icons/IconCheckCircle.svelte";
  import IconExclamationTriangle from "../common/icons/IconExclamationTriangle.svelte";

  type NoteSaveStatusProps = {
    status: AutosaveStatus;
    /** When the note was last saved, if it exists. */
    savedAt?: Date;
  };
  const { status, savedAt }: NoteSaveStatusProps = $props();
</script>

<!-- Not a live region: it changes at every pause while typing. Only failures are announced. -->
<p class="status">
  {#if status === "pending" || status === "saving"}
    Saving…
  {:else if status === "error"}
    <span class="error" role="alert">
      <IconExclamationTriangle class="text-red-400" />
      Not saved
    </span>
  {:else if savedAt}
    <IconCheckCircle class="text-slate-500" />
    <span title={formatFullDateTime(savedAt)}>Saved · {formatTime(savedAt)}</span>
  {:else}
    <span class="text-slate-500">Saves as you type</span>
  {/if}
</p>

<style lang="postcss">
  @reference "tailwindcss";

  .status {
    @apply flex items-center gap-1.5 text-xs text-slate-400 tabular-nums;
  }

  .error {
    @apply flex items-center gap-1.5 font-medium text-red-300;
  }
</style>
