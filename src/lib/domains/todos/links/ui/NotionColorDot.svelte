<script lang="ts">
import type { NotionColor } from "../model/types";

/** A small disc in one of Notion's colors, e.g. before a tag's name. */
type NotionColorDotProps = {
  color: NotionColor;
};
const { color }: NotionColorDotProps = $props();
</script>

<span class={["dot", `c-${color}`]} aria-hidden="true"></span>

<style lang="postcss">
  @reference "tailwindcss";

  .dot {
    @apply size-2 shrink-0 rounded-full bg-slate-400;

    &.c-gray {
      @apply bg-zinc-400;
    }
    &.c-brown {
      @apply bg-[#d4a27f];
    }
    &.c-orange {
      @apply bg-orange-400;
    }
    &.c-yellow {
      @apply bg-yellow-400;
    }
    &.c-green {
      @apply bg-green-400;
    }
    &.c-blue {
      @apply bg-sky-400;
    }
    &.c-purple {
      @apply bg-purple-400;
    }
    &.c-pink {
      @apply bg-pink-400;
    }
    &.c-red {
      @apply bg-red-400;
    }
  }
</style>
