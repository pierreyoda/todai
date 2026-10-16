<script lang="ts">
import IconExclamationTriangle from "#shared/ui/icons/IconExclamationTriangle.svelte";
import type { LinkStatus } from "../model/statuses";

type LinkStatusWordProps = {
  status: LinkStatus;
  /**
   * How it's shown, colored by its tone:
   * - `chip`: after a chip's label, with a glyph telling its tone;
   * - `pill`: on its own, e.g. in a link's details;
   * - `word`: within a sentence.
   */
  look: "chip" | "pill" | "word";
};
const { status, look }: LinkStatusWordProps = $props();

/** Its class, by look: the one each kept when they were styled where shown. */
const LOOK_CLASSES = { chip: "status", pill: "pill", word: "word" } as const;
</script>

{#snippet glyph()}
  {#if status.tone === "good"}
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M3.5 8.5 6.5 11.5 12.5 4.5" />
    </svg>
  {:else if status.tone === "bad"}
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
      <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
    </svg>
  {:else if status.tone === "pending"}
    <span class="ring" aria-hidden="true"></span>
  {:else if status.tone === "warn"}
    <IconExclamationTriangle />
  {/if}
{/snippet}

<!-- No whitespace around the word: within a sentence, it would show -->
<span class={[LOOK_CLASSES[look], status.tone]}>{#if look === "chip"}{@render glyph()}{/if}{status.word}</span>

<style lang="postcss">
  @reference "tailwindcss";

  .status {
    @apply inline-flex shrink-0 items-center gap-1 text-slate-400;

    > :global(svg) {
      @apply size-3;
    }
  }

  .ring {
    @apply size-1.5 rounded-full border-[1.5px] border-current;
  }

  .pill {
    @apply inline-flex items-center rounded-full bg-white/5 px-2 py-0.5 text-xs/4 font-semibold text-slate-300 ring-1 ring-white/15 ring-inset;
  }

  .word {
    @apply font-medium;
  }

  .status,
  .word {
    &.open,
    &.good {
      @apply text-green-300;
    }
    &.bad {
      @apply text-red-300;
    }
    &.pending,
    &.warn {
      @apply text-amber-300;
    }
    &.merged {
      @apply text-purple-300;
    }
  }

  .pill {
    &.open,
    &.good {
      @apply bg-green-500/15 text-green-300 ring-green-500/35;
    }
    &.bad {
      @apply bg-red-500/15 text-red-300 ring-red-500/35;
    }
    &.pending,
    &.warn {
      @apply bg-amber-400/12 text-amber-300 ring-amber-400/30;
    }
    &.merged {
      @apply bg-purple-500/15 text-purple-300 ring-purple-500/35;
    }
  }
</style>
