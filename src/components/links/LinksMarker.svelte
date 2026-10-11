<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { activeWorkspaceQueryOptions } from "../../client/queries";
  import type { TodoLink } from "../../client/types";
  import { formatLinksCount } from "../../utils/links";
  import { itemState, linkStatus } from "../../utils/linkStatuses";
  import { computePosition } from "../common/dropdown/positioning";
  import Button from "../common/Button.svelte";
  import IconLink from "../common/icons/IconLink.svelte";
  import LinkChip from "./LinkChip.svelte";

  /** A todo's links, in little: their count, then their list on hover or click. */
  type LinksMarkerProps = {
    links: readonly TodoLink[];
    /** The todo's title, which names the list. */
    title: string;
  };
  const { links, title }: LinksMarkerProps = $props();

  /** How long hovering takes to show the list, and leaving it to hide it. */
  const HOVER_OPEN_MS = 150;
  const HOVER_CLOSE_MS = 200;

  const panelId = $props.id();
  // With their status, as last fetched, through the workspace's connection: the calendar never fetches them
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const connected = $derived(!!activeWorkspace.data?.githubLogin);
  const allOnGitHub = $derived(links.every((link) => link.github));

  let open = $state(false);
  let button = $state<HTMLButtonElement>();
  let panel = $state<HTMLDivElement>();

  let timer: ReturnType<typeof setTimeout> | undefined;
  const later = (action: () => void, delay: number) => {
    clearTimeout(timer);
    timer = setTimeout(action, delay);
  };
  $effect(() => () => clearTimeout(timer));

  const position = () => {
    if (!button || !panel) return;
    const { x, y } = computePosition({
      anchor: button.getBoundingClientRect(),
      floating: panel.getBoundingClientRect(),
      viewport: {
        width: document.documentElement.clientWidth,
        height: document.documentElement.clientHeight,
      },
      placement: "bottom-start",
      offset: 4,
    });
    panel.style.left = `${x}px`;
    panel.style.top = `${y}px`;
  };

  // A manual popover, so that hovering it leaves any other one open (e.g. a menu)
  $effect(() => {
    if (!panel) return;
    const shown = panel.matches(":popover-open");
    if (open && !shown) {
      panel.showPopover();
      // Positioned in the same task as the opening, so it never paints at the default position
      position();
    } else if (!open && shown) {
      panel.hidePopover();
    }
  });

  // Closed by Escape or by clicking elsewhere, as it's not light-dismissed
  $effect(() => {
    if (!open) return;
    const onpointerdown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!button?.contains(target) && !panel?.contains(target)) open = false;
    };
    const onkeydown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      const hadFocus = panel?.contains(document.activeElement);
      open = false;
      if (hadFocus) button?.focus();
    };
    document.addEventListener("pointerdown", onpointerdown, true);
    document.addEventListener("keydown", onkeydown);
    window.addEventListener("resize", position);
    window.addEventListener("scroll", position, { capture: true, passive: true });
    return () => {
      document.removeEventListener("pointerdown", onpointerdown, true);
      document.removeEventListener("keydown", onkeydown);
      window.removeEventListener("resize", position);
      window.removeEventListener("scroll", position, { capture: true });
    };
  });
</script>

<!-- Hovering the marker or its list keeps it shown: the list is part of the group, though shown on top -->
<span
  class="group"
  role="presentation"
  onpointerenter={(event) => {
    if (event.pointerType === "mouse") later(() => (open = true), HOVER_OPEN_MS);
  }}
  onpointerleave={(event) => {
    if (event.pointerType === "mouse") later(() => (open = false), HOVER_CLOSE_MS);
  }}
>
  <Button
    bind:ref={button}
    style="plain"
    size="xs"
    class={["marker", open && "open"]}
    aria-label={formatLinksCount(links.length)}
    aria-expanded={open}
    aria-controls={panelId}
    onclick={(event) => {
      clearTimeout(timer);
      // Clicked once hovered: kept shown. With the keyboard: shown or hidden.
      open = event.detail > 0 ? true : !open;
    }}
  >
    <IconLink />
    {#if links.length > 1}
      <span aria-hidden="true">{links.length}</span>
    {/if}
  </Button>
  <div
    bind:this={panel}
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
