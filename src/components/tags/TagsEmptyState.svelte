<script lang="ts">
  import { scale } from "svelte/transition";
  import { backOut } from "svelte/easing";

  import IconTag from "../common/icons/IconTag.svelte";
  import IconPlus from "../common/icons/IconPlus.svelte";

  type TagsEmptyStateProps = {
    onCreate: () => void;
  };
  const { onCreate }: TagsEmptyStateProps = $props();
</script>

<button
  type="button"
  class="empty-tags"
  onclick={onCreate}
  in:scale={{ duration: 300, start: 0.9, easing: backOut }}
>
  <!-- Nested, as both the float and the hover lift use a transform -->
  <span class="icon-float">
    <span class="icon-wrapper">
      <IconTag class="size-6" />
      <span class="plus-badge">
        <IconPlus class="size-3 stroke-[2.5]" />
      </span>
    </span>
  </span>
  <span class="title">Create your first tag</span>
  <span class="hint">Group and filter your todos</span>
</button>

<style lang="postcss">
  @reference "tailwindcss";

  .empty-tags {
    /* Layout */
    @apply flex w-full cursor-pointer flex-col items-center gap-1 rounded-xl px-3 py-5;
    /* Surface: dashed, like a slot waiting to be filled */
    @apply border border-dashed border-white/15 bg-white/2.5;
    /* States */
    @apply transition duration-200 hover:-translate-y-0.5 hover:border-pink-500/60 hover:bg-pink-500/5 active:translate-y-0;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
    @apply motion-reduce:transition-none motion-reduce:hover:translate-y-0;

    &:hover .icon-wrapper {
      @apply text-pink-300 ring-pink-500/50;
    }
    &:hover .plus-badge {
      @apply rotate-90;
    }
  }

  .icon-float {
    @apply mb-2;
    animation: float 3s ease-in-out infinite;
    @apply motion-reduce:animate-none;
  }

  .icon-wrapper {
    @apply relative isolate flex size-12 items-center justify-center rounded-full;
    @apply bg-white/5 text-slate-300 ring-1 ring-white/10 ring-inset transition duration-200;
    /* Soft halo, breathing outwards (kept within the section's padding, which clips overflow) */
    &::before {
      content: "";
      @apply absolute inset-0 -z-10 rounded-full bg-pink-500/25;
      animation: halo 3s ease-out infinite;
      @apply motion-reduce:hidden;
    }
  }

  .plus-badge {
    @apply absolute -right-0.5 -bottom-0.5 flex size-5 items-center justify-center rounded-full;
    /* Cut out of the side panel by a ring of its color */
    @apply bg-pink-500 text-white shadow-md ring-2 ring-(--side-panel-bg) transition-transform duration-300;
  }

  .title {
    @apply text-sm font-medium text-white;
  }

  .hint {
    @apply text-xs text-slate-400;
  }

  @keyframes float {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-4px);
    }
  }

  @keyframes halo {
    0% {
      transform: scale(1);
      opacity: 0.8;
    }
    70%,
    100% {
      transform: scale(1.4);
      opacity: 0;
    }
  }
</style>
