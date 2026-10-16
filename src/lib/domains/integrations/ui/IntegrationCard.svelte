<script lang="ts">
import type { Snippet } from "svelte";
import Button from "#shared/ui/Button.svelte";

/**
 * A provider's card, in the integrations settings: its name, then how the workspace is connected to it. What's
 * particular to the provider comes from its own card (e.g. `GitHubCard`), which may lay out its body with the classes
 * `details` (a `<dl>`), `problem` (with `problem-title` and `problem-text`), `actions` and `unconnected`.
 */
type IntegrationCardProps = {
  /** E.g. "GitHub". */
  name: string;
  /** What it shows of the links' items, e.g. "Pull requests and issues: state, checks and reviews". */
  subtitle: string;
  logo: Snippet;
  /**
   * How the workspace is connected to it:
   * - `connected`;
   * - `attention`: connected, but its connection needs the user;
   * - `checking`: not known yet;
   * - `not_connected`: offers to connect it.
   */
  state: "connected" | "attention" | "checking" | "not_connected";
  /** Starts connecting the workspace, e.g. by opening a modal. */
  onConnect: () => void;
  children: Snippet;
};
const {
  name,
  subtitle,
  logo,
  state,
  onConnect,
  children,
}: IntegrationCardProps = $props();

const titleId = $props.id();
</script>

<article
  class={["card", state === "attention" && "attention"]}
  aria-labelledby={titleId}
>
  <header class="header">
    <span class="logo">{@render logo()}</span>
    <div class="heading">
      <h3 id={titleId} class="name">{name}</h3>
      <p class="subtitle">{subtitle}</p>
    </div>
    {#if state === "connected"}
      <span class="badge connected">Connected</span>
    {:else if state === "attention"}
      <span class="badge warning">Needs attention</span>
    {:else if state === "checking"}
      <span class="badge" role="status">Checking…</span>
    {:else}
      <Button onclick={onConnect}>Connect {name}</Button>
    {/if}
  </header>

  {@render children()}
</article>

<style lang="postcss">
  @reference "tailwindcss";

  .card {
    @apply flex flex-col gap-3 rounded-xl bg-white/3 p-4 ring-1 ring-white/10;

    &.attention {
      @apply ring-amber-400/35;
    }
  }

  .header {
    @apply flex items-center gap-3;
  }

  .logo {
    @apply grid size-9 shrink-0 place-items-center rounded-lg bg-white/6 text-white ring-1 ring-white/10 ring-inset;

    > :global(svg) {
      @apply size-4.5;
    }
  }

  .heading {
    @apply min-w-0 flex-1;
  }

  .name {
    @apply text-[15px]/[22px] font-semibold text-white;
  }

  .subtitle {
    @apply text-xs/4 text-slate-400;
  }

  .badge {
    @apply inline-flex shrink-0 items-center gap-1.5 rounded-full bg-white/5 px-2.5 text-xs/5 font-medium text-slate-300;

    &::before {
      @apply size-1.5 rounded-full bg-current content-[''];
    }

    &.connected {
      @apply bg-green-500/12 text-green-300;
    }

    &.warning {
      @apply bg-amber-400/12 text-amber-300;
    }
  }

  /* The provider's body: another component's elements */
  .card {
    :global(.unconnected) {
      @apply text-sm/5 text-slate-400;
    }

    :global(.details) {
      @apply grid grid-cols-[7.5rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-[13px]/5;

      :global(dt) {
        @apply text-slate-400;
      }

      :global(dd) {
        @apply text-white;
      }
    }

    :global(.problem) {
      @apply flex flex-col gap-0.5 rounded-lg bg-amber-400/8 p-3 ring-1 ring-amber-400/30;
    }

    :global(.problem-title) {
      @apply text-sm/5 font-medium text-white;
    }

    :global(.problem-text) {
      @apply text-[13px]/5 text-slate-300;
    }

    :global(code) {
      @apply rounded bg-white/5 px-1 py-0.5 font-mono text-xs text-slate-200;
    }

    :global(.actions) {
      @apply flex items-center gap-2;
    }
  }
</style>
