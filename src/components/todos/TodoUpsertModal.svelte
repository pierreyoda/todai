<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import Modal from "../common/Modal.svelte";
  import { invokeClient } from "../../client";
  import type { Todo } from "../../client/types";
  import TodoForm, { type TodoFormData } from "./TodoForm.svelte";
  import { invalidateTodosOf } from "../../client/queries";

  type TodoUpsertModalProps = {
    /** Bindable. */
    show: boolean;
  } & (
    | {
        existing: Todo;
      }
    | {
        existing?: never;
      }
  );
  let { show = $bindable(), existing }: TodoUpsertModalProps = $props();

  const queryClient = useQueryClient();

  // Invalidated through the todo's day, which refreshes its month (with its days' lists) and the per-month
  // statistics: the todos panel lists a day, but the calendar lists whole months.
  const createTodo = createMutation(() => ({
    mutationFn: ({ day, title }: TodoFormData) =>
      invokeClient({
        name: "create_todo",
        args: { day, title },
      }),
    onSuccess: (todo) => {
      show = false;
      return invalidateTodosOf(queryClient, todo.day);
    },
  }));
  const updateTodo = createMutation(() => ({
    mutationFn: ({ todo, data }: { todo: Todo; data: TodoFormData }) =>
      invokeClient({
        name: "update_todo",
        args: { params: { id: todo.id, ...data } },
      }),
    onSuccess: (_, { todo, data }) => {
      show = false;
      // A todo moved to another day leaves its former one
      const days = data.day === todo.day ? [todo.day] : [todo.day, data.day];
      return Promise.all(
        days.map((day) => invalidateTodosOf(queryClient, day)),
      );
    },
  }));
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      {#if existing}
        Edit an existing todo
      {:else}
        Create a new todo
      {/if}
    {/snippet}
    {#if existing}
      <TodoForm
        data={{
          day: existing.day,
          completed: existing.completed,
          title: existing.title,
        }}
        onSubmit={(data) => updateTodo.mutate({ todo: existing, data })}
      />
    {:else}
      <TodoForm onSubmit={(data) => createTodo.mutate(data)} />
    {/if}
  </Modal>
</div>
