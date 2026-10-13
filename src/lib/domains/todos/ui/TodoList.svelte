<script lang="ts">
import type { Day } from "#core/time";
import type { Tag } from "#domains/tags";
import { createReorderTodoMutation } from "../api/mutations";
import { reorderTodos, sameOrder } from "../model/order";
import type { Placement, Todo } from "../model/types";
import TodoLineItem from "./TodoLineItem.svelte";

type TodoListProps = {
  /** The todos of `day` shown, in display order. */
  todos: readonly Todo[];
  day: Day;
  tags: readonly Tag[];
  /** Whether they can be reordered by dragging them: only when all of them are shown. */
  reorderable?: boolean;
};

const { todos, day, tags, reorderable = false }: TodoListProps = $props();

const completedTodos = $derived(todos.filter((todo) => todo.completed));
const incompleteTodos = $derived(todos.filter((todo) => !todo.completed));

const reorderTodo = createReorderTodoMutation(() => day);
/** Puts the todo dropped on `target` right before or after it, among the todos as completed as it. */
const reorder = (target: Todo, droppedId: Todo["id"], placement: Placement) => {
  const todo = todos.find(({ id }) => id === droppedId);
  if (!todo || todo.completed !== target.completed) return;
  // E.g. dropped on itself, or right after the one before it
  if (sameOrder(todos, reorderTodos(todos, todo.id, target.id, placement)))
    return;
  reorderTodo.mutate({ todo, targetId: target.id, placement });
};
</script>

<!-- Two lists, as each one is reordered on its own -->
<div class="container">
  {#if incompleteTodos.length > 0}
    <ul class="list" aria-label="To do">
      {#each incompleteTodos as item (item.id)}
        <TodoLineItem
          {item}
          {tags}
          onReorder={reorderable ? (id, placement) => reorder(item, id, placement) : undefined}
        />
      {/each}
    </ul>
  {/if}
  {#if incompleteTodos.length > 0 && completedTodos.length > 0}
    <div class="divider" role="separator"></div>
  {/if}
  {#if completedTodos.length > 0}
    <ul class="list" aria-label="Completed">
      {#each completedTodos as item (item.id)}
        <TodoLineItem
          {item}
          {tags}
          onReorder={reorderable ? (id, placement) => reorder(item, id, placement) : undefined}
        />
      {/each}
    </ul>
  {/if}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full h-full flex flex-col gap-1.5 p-2 overflow-y-auto;
  }

  .list {
    @apply flex flex-col gap-1.5;
  }

  .divider {
    @apply w-full h-px border-b border-slate-400 py-2 mb-4;
  }
</style>
