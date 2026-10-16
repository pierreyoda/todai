<script lang="ts">
import { openLink } from "../api/openLink";
import { describeLink } from "../model/links";
import type { LinkStatus } from "../model/statuses";
import type { ItemState, ParsedLink } from "../model/types";
import LinkKindIcon from "./LinkKindIcon.svelte";
import LinkStatusWord from "./LinkStatusWord.svelte";

type LinkChipProps = {
  /** A todo's link, one about to be added, or one mentioned by a pull request or issue. */
  link: ParsedLink;
  /** Where its item stands, once known (e.g. a merged pull request): picks and colors its icon. */
  state?: ItemState;
  /** Its status word, after its label: `"loading"` while fetched for the first time. */
  status?: LinkStatus | "loading" | null;
  /** Opens its details, instead of opening the link in the browser. */
  onclick?: (event: MouseEvent) => void;
  /** Whether its details are open. */
  active?: boolean;
  /** Bindable: the chip's element, e.g. to anchor its details to. */
  ref?: HTMLElement;
};
let {
  link,
  state,
  status,
  onclick,
  active = false,
  ref = $bindable(),
}: LinkChipProps = $props();

const title = $derived(
  [
    describeLink(link),
    typeof status === "object" && status ? ` · ${status.word}` : "",
    onclick ? "" : ": opens in your browser",
  ].join(""),
);
</script>

{#snippet content()}
  <span class="icon"><LinkKindIcon kind={link.kind} {state} /></span>
  <span class={["label", typeof status === "object" && status?.tone]}>{link.label}</span>
  {#if status}
    <span class="divider" aria-hidden="true"></span>
    {#if status === "loading"}
      <span class="skeleton" aria-hidden="true"></span>
      <span class="sr-only">Loading its status</span>
    {:else}
      <LinkStatusWord {status} look="chip" />
    {/if}
  {/if}
{/snippet}

{#if onclick}
  <button
    bind:this={ref}
    type="button"
    class={["chip", active && "active"]}
    {title}
    aria-haspopup="dialog"
    aria-expanded={active}
    {onclick}
  >
    {@render content()}
  </button>
{:else}
  <!-- A link rather than a button, so that it reads as one; opened in the browser rather than in the app -->
  <a
    bind:this={ref}
    class="chip"
    href={link.url}
    {title}
    onclick={(event) => {
      event.preventDefault();
      openLink(link.url);
    }}
  >
    {@render content()}
  </a>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .chip {
    /* Layout: as tall as a line of tags, never squeezed by the ones next to it */
    @apply inline-flex h-5.5 max-w-72 shrink-0 cursor-pointer items-center gap-1.5 rounded-md pr-2 pl-1.5;
    /* Surface */
    @apply bg-white/5 ring-1 ring-white/10 ring-inset;
    /* Typography */
    @apply text-xs/4 font-medium whitespace-nowrap text-white tabular-nums;
    /* States */
    @apply transition-colors hover:bg-white/10;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;

    &.active {
      @apply bg-white/12 ring-pink-400/70;
    }
  }

  .icon {
    @apply grid shrink-0 text-slate-400;

    > :global(svg) {
      @apply size-3.5;
    }
  }

  .label {
    @apply min-w-0 truncate;

    &.warn {
      @apply text-slate-300;
    }
  }

  .divider {
    @apply h-3 w-px shrink-0 bg-white/12;
  }

  .skeleton {
    @apply block h-2 w-13 shrink-0 rounded-full bg-white/12 motion-safe:animate-pulse;
  }
</style>
