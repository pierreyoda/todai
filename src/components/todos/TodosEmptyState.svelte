<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  import type { Tag } from "../../client/types";
  import Button from "../common/Button.svelte";
  import IconSun from "../common/icons/IconSun.svelte";
  import IconTag from "../common/icons/IconTag.svelte";

  type TodosEmptyStateProps = {
    /** Set when the day has todos, but none with this tag. */
    filterTag?: Tag | null;
    onClearFilter?: () => void;
  };
  const { filterTag, onClearFilter }: TodosEmptyStateProps = $props();
</script>

<div
  class="empty-todos"
  style:--accent={filterTag?.color}
  role="status"
  in:fly={{ y: 16, duration: 400, easing: cubicOut }}
>
  <div class="illustration" aria-hidden="true">
    <span class="ripple"></span>
    <span class="ripple delayed"></span>
    <div class="disc">
      {#if filterTag}
        <IconTag class="size-14" />
      {:else}
        <IconSun class="spin size-16" />
      {/if}
    </div>
  </div>
  {#if filterTag}
    <h2 class="title">
      No todos tagged <span class="text-(--accent)">{filterTag.name}</span>
    </h2>
    <p class="hint">Other todos are planned today, just not with this tag.</p>
    {#if onClearFilter}
      <Button style="outline" class="mt-2" onclick={onClearFilter}>
        Show all todos
      </Button>
    {/if}
  {:else}
    <h2 class="title">A fresh, clear day</h2>
    <p class="hint">Nothing planned yet: add your first todo above.</p>
  {/if}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .empty-todos {
    /* Pink by default, the tag's color when filtering */
    --accent: var(--color-pink-400);
    @apply flex grow flex-col items-center justify-center gap-2 p-8 text-center;
  }

  .illustration {
    @apply relative mb-6 flex size-40 items-center justify-center;
  }

  /* Rings expanding outwards from the disc, staggered */
  .ripple {
    @apply absolute inset-6 rounded-full border border-(--accent) opacity-0;
    animation: ripple 4s ease-out infinite;
    @apply motion-reduce:hidden;
    &.delayed {
      animation-delay: 2s;
    }
  }

  .disc {
    @apply flex size-28 items-center justify-center rounded-full text-(--accent);
    @apply bg-[radial-gradient(circle,color-mix(in_oklab,var(--accent)_22%,transparent),transparent_70%)];
    @apply ring-1 ring-white/10 ring-inset;
    animation: glow 4s ease-in-out infinite;
    @apply motion-reduce:animate-none;
  }

  .disc :global(.spin) {
    animation: spin 24s linear infinite;
    @apply motion-reduce:animate-none;
  }

  .title {
    @apply text-xl font-semibold text-white;
  }

  .hint {
    @apply max-w-xs text-sm text-slate-400;
  }

  @keyframes ripple {
    0% {
      transform: scale(1);
      opacity: 0.6;
    }
    100% {
      transform: scale(1.5);
      opacity: 0;
    }
  }

  @keyframes glow {
    0%,
    100% {
      box-shadow: 0 0 24px -8px var(--accent);
    }
    50% {
      box-shadow: 0 0 48px -4px var(--accent);
    }
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
