<script lang="ts">
import { type Snippet, untrack } from "svelte";
import { type Provider, providerInfo } from "#core/integrations";
import Button from "#shared/ui/Button.svelte";
import { computePosition } from "#shared/ui/dropdown/positioning.ts";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import IconArrowPath from "#shared/ui/icons/IconArrowPath.svelte";
import IconArrowTopRightOnSquare from "#shared/ui/icons/IconArrowTopRightOnSquare.svelte";
import IconExclamationTriangle from "#shared/ui/icons/IconExclamationTriangle.svelte";
import { formatRelativeTime } from "#shared/utils/timestamps.ts";
import type { Todo } from "../../model/types";
import {
  createRefreshTodoLinkMutation,
  createRemoveTodoLinkMutation,
} from "../api/mutations";
import { openLink } from "../api/openLink";
import { describeProblem, linkStatusWords } from "../model/statuses";
import type { ItemState, LinkFetchError, TodoLink } from "../model/types";
import LinkKindIcon from "./LinkKindIcon.svelte";
import LinkStatusWord from "./LinkStatusWord.svelte";

/** A link's details, whatever its provider: what's particular to the provider comes from its own popover (e.g. `GitHubLinkPopover`). */
type LinkPopoverProps = {
  /** Bindable. */
  open: boolean;
  /** A link of `todo`, fetched from `provider`. */
  link: TodoLink;
  todo: Todo;
  provider: Provider;
  /** What it's shown next to: its chip. */
  anchor: HTMLElement | undefined;
  /** Where its item stands, once known: picks and colors its icon. */
  iconState?: ItemState;
  /** Where its item is, next to its icon: e.g. its repository. */
  where: Snippet;
  /** After its status words, e.g. a pull request's branches. */
  extras?: Snippet;
  /** Why its provider has nothing to tell about it, in a sentence. */
  fetchErrorNotice: Snippet<[LinkFetchError]>;
  /** Its details, once fetched: rows of a `<dl>` (see `LinkDetailsRow`). */
  details?: Snippet;
};
let {
  open = $bindable(),
  link,
  todo,
  provider,
  anchor,
  iconState,
  where,
  extras,
  fetchErrorNotice,
  details,
}: LinkPopoverProps = $props();

const titleId = $props.id();
const refresh = createRefreshTodoLinkMutation(() => todo);
const remove = createRemoveTodoLinkMutation(() => todo);

const info = $derived(providerInfo(provider));
const words = $derived(linkStatusWords(link));
const refreshProblem = $derived.by(() => {
  const problem = refresh.data?.problems[provider];
  return problem && describeProblem(provider, problem);
});

let popover = $state<HTMLDivElement>();

const position = () => {
  if (!popover || !anchor) return;
  const { x, y } = computePosition({
    anchor: anchor.getBoundingClientRect(),
    floating: popover.getBoundingClientRect(),
    viewport: {
      width: document.documentElement.clientWidth,
      height: document.documentElement.clientHeight,
    },
    placement: "bottom-start",
    offset: 6,
  });
  popover.style.left = `${x}px`;
  popover.style.top = `${y}px`;
};

// Follows `open`: shown as a popover, so that clicking outside or pressing Escape closes it
$effect(() => {
  if (!popover) return;
  const shown = popover.matches(":popover-open");
  if (open && !shown) {
    // What the last refresh ran into is only worth showing until closed
    untrack(() => refresh.reset());
    popover.showPopover({ source: anchor });
    // Positioned in the same task as the opening, so it never paints at the default position
    position();
    popover.focus({ preventScroll: true });
  } else if (!open && shown) {
    popover.hidePopover();
  }
});

// Kept next to its chip, which can move, e.g. when the list scrolls
$effect(() => {
  if (!open) return;
  window.addEventListener("resize", position);
  window.addEventListener("scroll", position, {
    capture: true,
    passive: true,
  });
  return () => {
    window.removeEventListener("resize", position);
    window.removeEventListener("scroll", position, { capture: true });
  };
});

/** Whether focus was within when it closed: then given back to its chip. */
let hadFocus = false;
</script>

<div
  bind:this={popover}
  popover="auto"
  role="dialog"
  tabindex="-1"
  aria-labelledby={titleId}
  class="popover"
  onbeforetoggle={(event) => {
    if (event.newState === "closed") {
      hadFocus = !!popover?.contains(document.activeElement);
    }
  }}
  ontoggle={(event) => {
    open = event.newState === "open";
    if (!open && hadFocus) anchor?.focus({ preventScroll: true });
  }}
>
  <header class="header">
    <div class="context">
      <LinkKindIcon kind={link.kind} state={iconState} />
      <span class="where">
        {@render where()}
      </span>
      <Button
        style="plain"
        size="xs"
        class="refresh"
        aria-label={`Refresh from ${info.name}`}
        title={`Refresh from ${info.name}`}
        disabled={refresh.isPending}
        onclick={() => refresh.mutate(link)}
      >
        <IconArrowPath
          class={[refresh.isPending && "motion-safe:animate-spin"]}
        />
      </Button>
    </div>
    <h2 id={titleId} class="title">{link.snapshot?.title ?? link.label}</h2>
    {#if words.length > 0}
      <div class="pills">
        {#each words as word (word.word)}
          <LinkStatusWord status={word} look="pill" />
        {/each}
        {@render extras?.()}
      </div>
    {/if}
  </header>

  {#if link.fetchError}
    <p class="notice" role="alert">
      <IconExclamationTriangle class="shrink-0" />
      {@render fetchErrorNotice(link.fetchError)}
    </p>
  {/if}
  {#if refreshProblem}
    <p class="notice" role="alert">
      <IconExclamationTriangle class="shrink-0" />
      {refreshProblem.detail}
    </p>
  {/if}
  {#if refresh.isError}
    <div class="px-3 pb-2">
      <ErrorBanner
        title="Could not refresh it"
        error={refresh.error}
        onDismiss={() => refresh.reset()}
      />
    </div>
  {/if}
  {#if remove.isError}
    <div class="px-3 pb-2">
      <ErrorBanner
        title="Could not remove the link"
        error={remove.error}
        onDismiss={() => remove.reset()}
      />
    </div>
  {/if}

  {#if details}
    <dl class="details">
      {@render details()}
    </dl>
  {/if}

  <footer class="footer">
    <!-- Announced as it's refreshed, then fetched -->
    <span class="fetched" role="status">
      {#if refresh.isPending}
        Refreshing…
      {:else if link.fetchedAt}
        Fetched {formatRelativeTime(link.fetchedAt).toLowerCase()}
      {:else}
        Not fetched yet
      {/if}
    </span>
    <Button
      style="plain"
      size="xs"
      class="remove"
      disabled={remove.isPending}
      onclick={() => remove.mutate(link, { onSuccess: () => (open = false) })}
    >
      Remove link
    </Button>
    <a
      class="open"
      href={link.url}
      onclick={(event) => {
        event.preventDefault();
        openLink(link.url);
      }}
    >
      Open {info.where}
      <IconArrowTopRightOnSquare class="size-3.5" />
    </a>
  </footer>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .popover {
    /* Reset popover defaults: positioned by script */
    @apply inset-auto m-0 border-0;
    /* Sizing: scrolls when taller than the viewport */
    @apply w-98 max-w-[calc(100vw-1rem)] max-h-[calc(100dvh-1rem)] overflow-y-auto overscroll-contain p-1;
    /* Surface */
    @apply rounded-xl bg-slate-800/92 text-sm/5 text-slate-300 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;
    @apply focus:outline-hidden;
  }

  .header {
    @apply flex flex-col gap-2.5 px-3 pt-3 pb-2.5;
  }

  .context {
    @apply flex items-center gap-2 text-slate-400;

    > :global(svg) {
      @apply size-4 shrink-0;
    }
  }

  .where {
    @apply min-w-0 flex-1 truncate text-xs/4;
  }

  /* A `Button`, restyled through its container as it's another component's element: more specific than its own rules */
  .context :global(.refresh) {
    @apply grid size-7 shrink-0 place-items-center rounded-md border-0 p-0 text-slate-400 transition-colors;
    @apply hover:text-white disabled:cursor-default disabled:opacity-100 focus-visible:outline-offset-0;

    /* Highlighted when hovered only, not when pressed */
    &:enabled:active {
      @apply bg-transparent;
    }
    &:enabled:hover {
      @apply hover:bg-white/10;
    }
  }

  .title {
    @apply text-[15px]/[22px] font-semibold text-white wrap-break-word;
  }

  .pills {
    @apply flex flex-wrap items-center gap-1.5;
  }

  .notice {
    @apply mx-3 mb-2 flex items-start gap-2 rounded-lg bg-amber-400/10 px-2.5 py-2 text-[13px]/5 text-amber-200;

    > :global(svg) {
      @apply mt-0.5 size-4;
    }
  }

  .details {
    @apply flex flex-col border-t border-white/6;

    /* `LinkDetailsRow`'s */
    > :global(.row + .row) {
      @apply border-t border-white/4;
    }
  }

  .footer {
    @apply flex items-center gap-2 border-t border-white/10 py-2 pr-2 pl-3;
  }

  .fetched {
    /* Dimmer than the rest, yet readable enough (WCAG AA) */
    @apply flex-1 text-xs/4 text-slate-400;
  }

  /* A `Button`, restyled as `.refresh` */
  .footer :global(.remove) {
    @apply rounded-md border-0 px-2 py-1 text-xs/4 font-medium text-red-400 transition-colors;
    @apply disabled:cursor-default disabled:opacity-50 focus-visible:outline-offset-0;

    /* Highlighted when hovered, even disabled, but not when pressed */
    &:enabled:active {
      @apply bg-transparent;
    }
    &:enabled:hover,
    &:disabled:hover {
      @apply hover:bg-red-500/10;
    }
  }

  .open {
    @apply inline-flex items-center gap-1.5 rounded-md bg-zinc-600 px-2.5 py-1.25 text-[13px]/[18px] font-semibold text-white shadow-[inset_0_1px_rgb(255_255_255/0.15)] transition-colors hover:bg-zinc-500;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }
</style>
