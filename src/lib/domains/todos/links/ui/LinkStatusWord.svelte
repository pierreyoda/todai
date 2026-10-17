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

/** The glyph shown on a chip: the status' own, else its tone's. */
const glyphOf = (): "check" | "cross" | "ring" | "dot" | "warn" | undefined => {
  if (status.glyph) return status.glyph;
  switch (status.tone) {
    case "good":
      return "check";
    case "bad":
      return "cross";
    case "pending":
      return "ring";
    case "warn":
      return "warn";
    default:
      return undefined;
  }
};
const glyph = $derived(glyphOf());
</script>

{#snippet glyphMark()}
  {#if glyph === "check"}
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M3.5 8.5 6.5 11.5 12.5 4.5" />
    </svg>
  {:else if glyph === "cross"}
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
      <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
    </svg>
  {:else if glyph === "ring"}
    <span class="ring" aria-hidden="true"></span>
  {:else if glyph === "dot"}
    <span class="dot" aria-hidden="true"></span>
  {:else if glyph === "warn"}
    <IconExclamationTriangle />
  {/if}
{/snippet}

<!-- No whitespace around the word: within a sentence, it would show -->
<span class={[LOOK_CLASSES[look], status.tone, status.color && `c-${status.color}`]}>{#if look === "chip"}{@render glyphMark()}{/if}{status.word}</span>

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

  .dot {
    @apply size-1.5 rounded-full bg-current;
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

  /* Notion's colors: over the tone's */
  .status,
  .word {
    &.c-default {
      @apply text-slate-300;
    }
    &.c-gray {
      @apply text-zinc-400;
    }
    &.c-brown {
      @apply text-[#d4a27f];
    }
    &.c-orange {
      @apply text-orange-300;
    }
    &.c-yellow {
      @apply text-yellow-300;
    }
    &.c-green {
      @apply text-green-300;
    }
    &.c-blue {
      @apply text-sky-300;
    }
    &.c-purple {
      @apply text-purple-300;
    }
    &.c-pink {
      @apply text-pink-300;
    }
    &.c-red {
      @apply text-red-300;
    }
  }

  .pill {
    &.c-gray {
      @apply bg-zinc-500/15 text-zinc-300 ring-zinc-500/35;
    }
    &.c-brown {
      @apply bg-[#d4a27f]/15 text-[#d4a27f] ring-[#d4a27f]/35;
    }
    &.c-orange {
      @apply bg-orange-500/15 text-orange-300 ring-orange-500/35;
    }
    &.c-yellow {
      @apply bg-yellow-500/15 text-yellow-300 ring-yellow-500/35;
    }
    &.c-green {
      @apply bg-green-500/15 text-green-300 ring-green-500/35;
    }
    &.c-blue {
      @apply bg-sky-500/15 text-sky-300 ring-sky-500/35;
    }
    &.c-purple {
      @apply bg-purple-500/15 text-purple-300 ring-purple-500/35;
    }
    &.c-pink {
      @apply bg-pink-500/15 text-pink-300 ring-pink-500/35;
    }
    &.c-red {
      @apply bg-red-500/15 text-red-300 ring-red-500/35;
    }
  }
</style>
