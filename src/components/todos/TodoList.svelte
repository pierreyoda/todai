<script lang="ts">
  import type { Todo } from "../../client/types";

  type TodoListProps = {
    todos: readonly Todo[];
  };

  const { todos }: TodoListProps = $props();
</script>

<ul class="container">
  {#each todos as todo (todo.id)}
    <li class={["todo-item", todo.completed && "completed"]}>
      <h3 class="title">{todo.title}</h3>
    </li>
  {/each}
</ul>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full h-full flex flex-col gap-3 p-2 overflow-y-auto;
  }

  .todo-item {
    @apply flex flex-col gap-1 px-4 py-3 rounded-lg;
    @apply border border-white/10 bg-white/5;
    @apply transition-colors hover:border-white/20 hover:bg-white/7.5;

    .title {
      @apply text-sm font-medium text-white wrap-break-word;
    }
  }
</style>
