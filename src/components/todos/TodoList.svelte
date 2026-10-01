<script lang="ts">
  import type { Todo } from "../../client/types";

  type TodoListProps = {
    todos: readonly Todo[];
  };

  const { todos }: TodoListProps = $props();
</script>

<div class="container">
  {#each todos as todo (todo.id)}
    <div class="todo-item">
      <h3>{todo.title}</h3>
      {#if todo.description}
        <p>{todo.description}</p>
      {/if}
    </div>
  {/each}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full h-full flex flex-col gap-2;
  }

  .todo-item {
    @apply p-4 bg-white rounded shadow;
  }
</style>
