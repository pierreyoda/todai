<script lang="ts">
  import type { PageProps } from "./$types";

  import TodoPanel from "../components/todos/TodoPanel.svelte";

  const { todos }: PageProps = $props();
</script>

<main>
  <TodoPanel {todos} />
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
