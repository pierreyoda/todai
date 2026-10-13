<script lang="ts">
  import type { GitHubProblem } from "#core/github";
  import type { TodoLink } from "../model/types";
  import {
    describeProblem,
    statusesFetchedAt,
  } from "../model/statuses";
  import IconExclamationTriangle from "#shared/ui/icons/IconExclamationTriangle.svelte";

  type LinkStatusesIndicatorProps = {
    /** Why GitHub couldn't be asked about the links' statuses. */
    problem: GitHubProblem;
    /** The links shown, for when their statuses are from. */
    links: readonly TodoLink[];
  };
  const { problem, links }: LinkStatusesIndicatorProps = $props();

  const described = $derived(
    describeProblem(problem, statusesFetchedAt(links)),
  );
  /** Fixed from the integrations settings, rather than by waiting. */
  const fixable = $derived(
    problem.kind === "signed_out" || problem.kind === "gh_missing",
  );
</script>

<span class="indicator" role="status">
  {#if fixable}
    <a
      href="/settings/integrations"
      class="pill attention"
      title={described.detail}
    >
      <IconExclamationTriangle />
      {described.label}
    </a>
  {:else}
    <span
      class={["pill", problem.kind !== "offline" && "attention"]}
      title={described.detail}
    >
      {#if problem.kind !== "offline"}
        <IconExclamationTriangle />
      {/if}
      {described.label}
    </span>
  {/if}
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .indicator {
    /* At the end of the toolbar */
    @apply ml-auto flex shrink-0;
  }

  .pill {
    @apply inline-flex items-center gap-1.5 rounded-full px-2.5 py-1;
    /* Surface */
    @apply bg-white/5 ring-1 ring-white/10 ring-inset;
    /* Typography */
    @apply text-xs/4 whitespace-nowrap text-slate-400;

    > :global(svg) {
      @apply size-3.5 shrink-0;
    }

    &.attention {
      @apply text-amber-300 ring-amber-400/30;
    }
  }

  a.pill {
    @apply transition-colors hover:bg-white/10;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }
</style>
