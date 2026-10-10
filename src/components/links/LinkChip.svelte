<script lang="ts">
  import type { ParsedLink } from "../../client/types";
  import { describeLink } from "../../utils/links";
  import { openLink } from "../../utils/openLink";
  import LinkKindIcon from "./LinkKindIcon.svelte";

  type LinkChipProps = {
    /** A todo's link, or one about to be added. */
    link: ParsedLink;
  };
  const { link }: LinkChipProps = $props();
</script>

<!-- A link rather than a button, so that it reads as one; opened in the browser rather than in the app -->
<a
  class="chip"
  href={link.url}
  title={`${describeLink(link)}: opens in your browser`}
  onclick={(event) => {
    event.preventDefault();
    openLink(link.url);
  }}
>
  <LinkKindIcon kind={link.kind} />
  <span class="label">{link.label}</span>
</a>

<style lang="postcss">
  @reference "tailwindcss";

  .chip {
    /* Layout: as tall as a line of tags, never squeezed by the ones next to it */
    @apply inline-flex h-5.5 max-w-48 shrink-0 items-center gap-1.5 rounded-md pr-2 pl-1.5;
    /* Surface */
    @apply bg-white/5 ring-1 ring-white/10 ring-inset;
    /* Typography */
    @apply text-xs/4 font-medium text-white tabular-nums;
    /* States */
    @apply transition-colors hover:bg-white/10;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;

    > :global(svg) {
      @apply size-3.5 shrink-0 text-slate-400;
    }
  }

  .label {
    @apply min-w-0 truncate;
  }
</style>
