<script lang="ts">
  import type { Snippet } from "svelte";

  type DropdownSectionProps = {
    heading?: string;
    children: Snippet;
  };
  const { heading, children }: DropdownSectionProps = $props();

  const headingId = $props.id();
</script>

<div role="group" aria-labelledby={heading ? headingId : undefined}>
  {#if heading}
    <div id={headingId} class="heading">{heading}</div>
  {/if}
  {@render children()}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .heading {
    @apply px-3.5 pt-2 pb-1 text-sm/5 font-medium text-slate-400 select-none sm:px-3 sm:text-xs/5;
  }
</style>
