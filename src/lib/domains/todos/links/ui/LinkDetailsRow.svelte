<script lang="ts">
import type { Snippet } from "svelte";
import type { ClassValue } from "svelte/elements";

/** A row of a link's details, in its popover: within a `<dl>`. */
type LinkDetailsRowProps = {
  label: string;
  /** Of its value. */
  class?: ClassValue;
  children: Snippet;
};
const { label, class: valueClass, children }: LinkDetailsRowProps = $props();
</script>

<div class="row">
  <dt>{label}</dt>
  <dd class={valueClass}>{@render children()}</dd>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  /* Separated from the previous one by its popover (see `LinkPopover`): siblings are another instance's */
  .row {
    @apply grid grid-cols-[5.5rem_minmax(0,1fr)] gap-3 px-3 py-2;

    dt {
      @apply text-xs/5 text-slate-400;
    }

    dd {
      @apply text-[13px]/5 text-white;
    }
  }
</style>
