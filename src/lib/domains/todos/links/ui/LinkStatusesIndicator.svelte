<script lang="ts">
import {
  needsUser,
  type Provider,
  type ProviderProblems,
  problemEntries,
} from "#core/integrations";
import IconExclamationTriangle from "#shared/ui/icons/IconExclamationTriangle.svelte";
import { linkProvider } from "../model/links";
import { describeProblem, statusesFetchedAt } from "../model/statuses";
import type { TodoLink } from "../model/types";

type LinkStatusesIndicatorProps = {
  /** Why providers couldn't be asked about the links' statuses. */
  problems: ProviderProblems;
  /** The links shown, for when their statuses are from. */
  links: readonly TodoLink[];
};
const { problems, links }: LinkStatusesIndicatorProps = $props();

const linksOf = (provider: Provider) =>
  links.filter((link) => linkProvider(link) === provider);

const described = $derived(
  problemEntries(problems).map(([provider, problem]) => ({
    provider,
    problem,
    ...describeProblem(provider, problem, statusesFetchedAt(linksOf(provider))),
  })),
);
</script>

<span class="indicator" role="status">
  {#each described as { provider, problem, label, detail } (provider)}
    <!-- Fixed from the integrations settings, rather than by waiting -->
    {#if needsUser(problem)}
      <a href="/settings/integrations" class="pill attention" title={detail}>
        <IconExclamationTriangle />
        {label}
      </a>
    {:else}
      <span
        class={["pill", problem.kind !== "offline" && "attention"]}
        title={detail}
      >
        {#if problem.kind !== "offline"}
          <IconExclamationTriangle />
        {/if}
        {label}
      </span>
    {/if}
  {/each}
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .indicator {
    /* At the end of the toolbar */
    @apply ml-auto flex shrink-0 gap-2;
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
