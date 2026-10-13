<script lang="ts">
  import type { Snippet } from "svelte";

  import IconSpinner from "./icons/IconSpinner.svelte";

  type LoadingStateProps = {
    state: "loading" | "error";
    loading?: Snippet;
    error?: Snippet;
  };
  const { state, loading, error }: LoadingStateProps = $props();
</script>

{#if state === "loading"}
  <div class="loading-state" role="status">
    <!-- Nested, as both animations use a transform -->
    <div class="bounce">
      <IconSpinner
        class="size-12 animate-spin text-pink-400 motion-reduce:animate-none"
      />
    </div>
    <p class="message">
      {#if loading}
        {@render loading()}
      {:else}
        Loading...
      {/if}
    </p>
  </div>
{:else}
  <div class="loading-state" role="alert">
    <p class="message error">
      {#if error}
        {@render error()}
      {:else}
        Error.
      {/if}
    </p>
  </div>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .loading-state {
    @apply flex grow flex-col items-center justify-center gap-4 self-stretch;
  }

  .bounce {
    animation: bounce-soft 1s ease-in-out infinite;
    @apply motion-reduce:animate-none;
  }

  /* Slighter than Tailwind's `animate-bounce` */
  @keyframes bounce-soft {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-15%);
    }
  }

  .message {
    @apply text-sm font-medium tracking-wide text-slate-400;
    &.error {
      @apply text-red-400;
    }
  }
</style>
