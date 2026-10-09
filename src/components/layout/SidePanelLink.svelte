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
  };
  const { label, icon, routeId, hint }: SidePanelLinkProps = $props();

  const current = $derived((page.route.id ?? "/") === routeId);
</script>

<a href={routeId} class="link" aria-current={current ? "page" : undefined}>
  <span class="icon">{@render icon()}</span>
  <span class="label">{label}</span>
  {#if hint}
    <span class="hint">{hint}</span>
  {/if}
</a>

<style lang="postcss">
  @reference "tailwindcss";

  .link {
    /* Layout: icons and labels line up with the tags below */
    @apply flex items-center gap-3 rounded-lg px-2.5 py-1.5;
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
</style>
