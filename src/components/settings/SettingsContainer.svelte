<script lang="ts">
  import type { Snippet } from "svelte";

  type SettingsContainerProps = {
    title: Snippet;
    children: Snippet;
  };
  const { title, children }: SettingsContainerProps = $props();
</script>

<section>
  <h2 class="title">{@render title()}</h2>
  <div class="content">
    {@render children()}
  </div>
</section>

<style lang="postcss">
  @reference "tailwindcss";

  section {
    @apply w-full h-full flex flex-col p-4;
  }

  .title {
    @apply text-center text-white text-lg;
  }

  .content {
    @apply flex-1 min-h-0 flex flex-col;
  }
</style>
