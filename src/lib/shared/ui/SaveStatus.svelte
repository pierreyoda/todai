<script lang="ts">
import type { AutosaveStatus } from "../utils/autosave.svelte";
import { formatFullDateTime, formatTime } from "../utils/timestamps";
import IconCheckCircle from "./icons/IconCheckCircle.svelte";
import IconExclamationTriangle from "./icons/IconExclamationTriangle.svelte";

/** Where an `Autosave` is, in a few words: e.g. next to what it saves. */
type SaveStatusProps = {
  status: AutosaveStatus;
  /** When what's saved was last saved, if it was. */
  savedAt?: Date;
};
const { status, savedAt }: SaveStatusProps = $props();
</script>

<!-- Not a live region: it changes at every pause while typing. Failures are announced by the page's banner. -->
<p class="status">
  {#if status === "pending" || status === "saving"}
    Saving…
  {:else if status === "error"}
    <span class="error">
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
