<script lang="ts">
  import type { Todo } from "../../client/types";
  import TodoLineItem from "./TodoLineItem.svelte";

  type TodoListProps = {
    todos: readonly Todo[];
  };

  const { todos }: TodoListProps = $props();

  const completedTodos = $derived(todos.filter((todo) => todo.completed));
  const incompleteTodos = $derived(todos.filter((todo) => !todo.completed));
</script>

<ul class="container">
  {#each incompleteTodos as item (item.id)}
    <TodoLineItem {item} />
  {/each}
  {#if incompleteTodos.length > 0 && completedTodos.length > 0}
    <div class="divider"></div>
  {/if}
  {#if completedTodos.length > 0}
    {#each completedTodos as item (item.id)}
      <TodoLineItem {item} />
    {/each}
  {/if}
</ul>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full h-full flex flex-col gap-1.5 p-2 overflow-y-auto;
  }

  .divider {
    @apply w-full h-px border-b border-slate-400 py-2 mb-4;
  }
</style>
