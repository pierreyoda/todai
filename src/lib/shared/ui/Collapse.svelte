<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassValue } from "svelte/elements";

  type CollapseProps = {
    /** Bindable. */
    open?: boolean;
    /** Content of the toggle button. */
    summary: Snippet;
    /** Wraps the toggle button in a heading of this level, so the section is listed in the outline. */
    headingLevel?: 2 | 3 | 4 | 5 | 6;
    class?: ClassValue;
    /**
     * Only created once the section is first opened (so e.g. its queries only run then), and kept afterwards
     * so that collapsing it again doesn't lose its state.
     */
    children: Snippet;
  };

  let {
    open = $bindable(false),
    summary,
    headingLevel,
    class: extraClass,
    children,
  }: CollapseProps = $props();

  const id = $props.id();
  const contentId = `${id}-content`;

  let hasBeenOpened = false;
  // Sticky: becomes true with `open`, then stays so
  const rendered = $derived((hasBeenOpened ||= open));
</script>

{#snippet toggle()}
  <button
    type="button"
    class="toggle"
    aria-expanded={open}
    aria-controls={contentId}
    onclick={() => (open = !open)}
  >
    <svg class="chevron" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path
        fill-rule="evenodd"
        d="M6.22 4.22a.75.75 0 0 1 1.06 0l3.25 3.25a.75.75 0 0 1 0 1.06l-3.25 3.25a.75.75 0 0 1-1.06-1.06L8.94 8 6.22 5.28a.75.75 0 0 1 0-1.06Z"
        clip-rule="evenodd"
      />
    </svg>
    <span class="summary">{@render summary()}</span>
  </button>
{/snippet}

<div class={["disclosure", open && "open", extraClass]}>
  {#if headingLevel}
    <svelte:element this={`h${headingLevel}`} class="heading">
      {@render toggle()}
    </svelte:element>
  {:else}
    {@render toggle()}
  {/if}
  <!-- Inert while collapsed: neither focusable nor exposed to assistive technologies -->
  <div id={contentId} class="content" inert={!open}>
    <div class="content-inner">
      {#if rendered}
        {@render children()}
      {/if}
    </div>
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .heading {
    @apply m-0 text-inherit;
  }

  .toggle {
    /* Layout */
    @apply flex w-full cursor-pointer items-center gap-2 rounded-lg px-2 py-1.5 text-left;
    /* Typography */
    @apply text-sm/6 font-semibold text-white;
    /* States */
    @apply hover:bg-white/5 focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }

  .chevron {
    @apply size-4 shrink-0 text-slate-400 transition-transform duration-200 motion-reduce:transition-none;
  }

  /* Only its own toggle: nested sections have their own state */
  .open > .toggle .chevron,
  .open > .heading .chevron {
    @apply rotate-90;
  }

  .summary {
    @apply min-w-0 flex-1;
  }

  /* Animates between collapsed and natural height without measuring it */
  .content {
    @apply grid grid-rows-[0fr] transition-[grid-template-rows] duration-200 ease-out motion-reduce:transition-none;
  }

  .open > .content {
    @apply grid-rows-[1fr];
  }

  .content-inner {
    @apply min-h-0 overflow-hidden;
  }
</style>
