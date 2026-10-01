<script lang="ts">
  import type { Todo } from "../../types";
  import TodoCreateSection from "./TodoCreateSection.svelte";
  import TodoList from "./TodoList.svelte";

  type TodoPanelProps = {
    todos: readonly Todo[];
  };

  const { todos }: TodoPanelProps = $props();
</script>

<section class="container">
  <TodoCreateSection />
  <TodoList {todos} />
</section>

<style lang="postcss">
  @reference "tailwindcss";

  section {
    @apply w-full h-full flex flex-col;
  }
</style>
