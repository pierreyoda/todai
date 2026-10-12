<script lang="ts">
  import { page } from "$app/state";
  import type { Snippet } from "svelte";

  import type { RouteId } from "$app/types";

  type SidePanelLinkProps = {
    label: string;
    /** Leading icon, colored by the link. */
    icon: Snippet;
    routeId: RouteId;
    /** Short trailing text, e.g. a date. */
    hint?: string;
    /** A sub-page of the link above it: slimmer, to sit in its indented list. */
    nested?: boolean;
    /** Shown as a dot after the label, e.g. "Today has a note": the text is for screen readers only. */
    indicator?: string;
    /** The dot's color: `warning` when something needs the user, e.g. a connection. */
    indicatorTone?: "accent" | "warning";
    /** Whether it's also current on its sub-pages, e.g. the calendar's on a day's note opened from it. */
    currentOnSubPages?: boolean;
  };
  const {
    label,
    icon,
    routeId,
    hint,
    nested = false,
    indicator,
    indicatorTone = "accent",
    currentOnSubPages = false,
  }: SidePanelLinkProps = $props();

  const current = $derived.by(() => {
    const id = page.route.id ?? "/";
    return id === routeId || (currentOnSubPages && id.startsWith(`${routeId}/`));
  });
</script>

<a href={routeId} class={["link", nested && "nested"]} aria-current={current ? "page" : undefined}>
  <span class="icon">{@render icon()}</span>
  <span class="label">{label}</span>
  {#if hint}
    <span class="hint">{hint}</span>
  {/if}
  {#if indicator}
    <span class={["indicator", indicatorTone]} title={indicator}></span>
    <span class="sr-only">({indicator})</span>
  {/if}
</a>

<style lang="postcss">
  @reference "tailwindcss";

  .link {
    /* Layout: icons and labels line up with the tags below. Positioned, to hold its text for screen readers. */
    @apply relative flex items-center gap-3 rounded-lg px-2.5 py-1.5;
    /* Typography */
    @apply text-sm/6 font-medium text-slate-300;
    /* States */
    @apply transition-colors hover:bg-white/5 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500;

    &:hover .icon {
      @apply text-slate-300;
    }

    /* Current page: lifted, with its icon in the accent color */
    &[aria-current="page"] {
      @apply bg-white/10 text-white shadow-[inset_0_1px_--theme(--color-white/5%)];
      .icon {
        @apply text-pink-400;
      }
    }

    &.nested {
      @apply py-1;
    }
  }

  .icon {
    @apply flex shrink-0 text-slate-400 transition-colors;
  }

  .label {
    @apply min-w-0 flex-1 truncate;
  }

  .hint {
    @apply shrink-0 text-xs font-medium text-slate-500 tabular-nums;
  }

  /* Where the hint would be, in the accent color */
  .indicator {
    @apply mr-1 size-1.5 shrink-0 rounded-full bg-pink-400;

    &.warning {
      @apply bg-amber-400;
    }
  }
</style>
