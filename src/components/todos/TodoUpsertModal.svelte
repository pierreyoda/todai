<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import Modal from "../common/Modal.svelte";
  import { invokeClient } from "../../client";
  import type { Todo } from "../../client/types";
  import TodoForm, { type TodoFormData } from "./TodoForm.svelte";
  import TodoDescriptionField from "./TodoDescriptionField.svelte";
  import TodoLinksEditor from "../links/TodoLinksEditor.svelte";
  import { invalidateTodosOf } from "../../client/queries";

  type TodoUpsertModalProps = {
    /** Bindable. */
    show: boolean;
  } & (
    | {
        existing: Todo;
        /** Whether to focus its description's editor once shown, e.g. when opened to edit it. */
        focusDescription?: boolean;
      }
    | {
        existing?: never;
        focusDescription?: never;
      }
  );
  let {
    show = $bindable(),
    existing,
    focusDescription = false,
  }: TodoUpsertModalProps = $props();

  const queryClient = useQueryClient();

  let descriptionField = $state<ReturnType<typeof TodoDescriptionField>>();
  /** Whether to close it, once its todo's description is saved. */
  const saveBeforeClosing = () =>
    descriptionField ? descriptionField.saveBeforeLeaving() : true;

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

<!-- An existing todo's: large, for its description, saved before closing -->
<Modal
  bind:show
  size={existing ? "lg" : "md"}
  onCloseRequest={existing ? saveBeforeClosing : undefined}
>
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
      hint="The description and links save as you go · Update saves the rest"
      onSubmit={async (data) => {
        // Closed once updated: its description is saved first
        if (await saveBeforeClosing()) updateTodo.mutate({ todo: existing, data });
      }}
    >
      {#snippet main()}
        <TodoDescriptionField
          bind:this={descriptionField}
          todo={existing}
          autofocus={focusDescription}
        />
      {/snippet}
      {#snippet fields()}
        <TodoLinksEditor todo={existing} />
      {/snippet}
    </TodoForm>
  {:else}
    <TodoForm onSubmit={(data) => createTodo.mutate(data)} />
  {/if}
</Modal>
