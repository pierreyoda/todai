<script lang="ts">
  import Modal from "#shared/ui/Modal.svelte";
  import type { Todo } from "../model/types";
  import { createAddTodoMutation, createUpdateTodoMutation } from "../api/mutations";
  import TodoForm from "./TodoForm.svelte";
  import TodoDescriptionField from "./TodoDescriptionField.svelte";
  import TodoLinksEditor from "../links/ui/TodoLinksEditor.svelte";

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

  let descriptionField = $state<ReturnType<typeof TodoDescriptionField>>();
  /** Whether to close it, once its todo's description is saved. */
  const saveBeforeClosing = () =>
    descriptionField ? descriptionField.saveBeforeLeaving() : true;

  const close = () => (show = false);
  const createTodo = createAddTodoMutation({ onSuccess: close });
  const updateTodo = createUpdateTodoMutation({ onSuccess: close });
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
        if (await saveBeforeClosing()) updateTodo.mutate({ todo: existing, changes: data });
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
