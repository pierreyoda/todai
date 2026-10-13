<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { activeWorkspaceQueryOptions } from "../../client/queries";
  import type { TodoLink } from "../../client/types";
  import { formatLinksCount } from "../../utils/links";
  import { itemState, linkStatus } from "../../utils/linkStatuses";
  import Button from "#shared/ui/Button.svelte";
  import { HoverPopover } from "#shared/ui/hoverPopover.svelte.ts";
  import IconLink from "#shared/ui/icons/IconLink.svelte";
  import LinkChip from "./LinkChip.svelte";

  /** A todo's links, in little: their count, then their list on hover or click. */
  type LinksMarkerProps = {
    links: readonly TodoLink[];
    /** The todo's title, which names the list. */
    title: string;
  };
  const { links, title }: LinksMarkerProps = $props();

  const panelId = $props.id();
  // With their status, as last fetched, through the workspace's connection: the calendar never fetches them
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const connected = $derived(!!activeWorkspace.data?.githubLogin);
  const allOnGitHub = $derived(links.every((link) => link.github));

  const popover = new HoverPopover();
</script>

<span
  class="group"
  role="presentation"
  onpointerenter={popover.onpointerenter}
  onpointerleave={popover.onpointerleave}
>
  <Button
    bind:ref={popover.button}
    style="plain"
    size="xs"
    class={["marker", popover.open && "open"]}
    aria-label={formatLinksCount(links.length)}
    aria-expanded={popover.open}
    aria-controls={panelId}
    onclick={popover.onclick}
  >
    <IconLink />
    {#if links.length > 1}
      <span aria-hidden="true">{links.length}</span>
    {/if}
  </Button>
  <div
    bind:this={popover.panel}
    id={panelId}
    popover="manual"
    role="dialog"
    aria-label={`Links of ${title}`}
    class="panel"
  >
    <span class="count">{formatLinksCount(links.length)}</span>
    <ul class="chips">
      {#each links as link (link.id)}
        <li class="flex">
          <LinkChip
            {link}
            state={connected && link.snapshot ? itemState(link.snapshot) : undefined}
            status={connected ? linkStatus(link) : null}
          />
        </li>
      {/each}
    </ul>
    <span class="hint">
      Click one to open it {allOnGitHub ? "on GitHub" : "in your browser"}
    </span>
  </div>
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .group {
    @apply flex shrink-0;
  }

  /* `Button`'s, muted, as tight as the title's text: styled through the group, as it's another component's element */
  .group :global(.marker) {
    @apply items-center gap-0.5 px-1 text-[11px]/4 font-medium text-slate-400 tabular-nums;
    @apply transition-colors hover:text-white;

    &:global(.open) {
      @apply bg-white/10 text-white;
    }

    > :global(svg) {
      @apply size-3;
    }
  }

  .panel {
    /* Reset popover defaults: positioned by script */
    @apply inset-auto m-0 border-0;
    @apply w-max max-w-80 flex-col items-start gap-1.5 p-2.5;
    @apply rounded-lg bg-slate-800/95 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;

    /* Only while shown: a display set otherwise would override the browser's hiding of closed popovers */
    &:popover-open {
      @apply flex;
    }
  }

  .count {
    @apply text-[11px]/4 font-semibold tracking-wide text-slate-400 uppercase;
  }

  .chips {
    @apply flex flex-col items-start gap-1;
  }

  .hint {
    /* Dimmer than the rest, yet readable enough (WCAG AA) */
    @apply text-[11px]/4 text-slate-400;
  }
</style>
