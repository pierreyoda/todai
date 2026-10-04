<script lang="ts" module>
  export type TodoFormData = Pick<Todo, "day" | "title" | "completed">;
</script>

<script lang="ts">
  import type { Todo } from "../../client/types";
  import { dateToTodaiDate, isTodaiDate } from "../../utils";
  import Button from "../common/Button.svelte";
  import DatePicker from "../common/DatePicker.svelte";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";
  import FieldText from "../common/FieldText.svelte";

  type TodoFormProps = (
    | {
        /** Editing an existing todo. */
        data: TodoFormData;
      }
    | {
        /** Creating a new todo. */
        data?: never;
      }
  ) & {
    onSubmit: (submittedData: TodoFormData) => void;
  };
  const { data, onSubmit }: TodoFormProps = $props();

  let editedDay = $state(data?.day ?? dateToTodaiDate(new Date()));
  let editedTitle = $state(data?.title ?? "");
  let editedCompleted = $state(data?.completed ?? false);
  const valid = $derived(
    isTodaiDate(editedDay) && editedTitle.trim().length > 0,
  );
</script>

<div class="container">
  <FieldText bind:value={editedTitle} label="Title" />
  <div class="flex items-center">
    <FieldCheckbox
      label="Completed"
      bind:checked={editedCompleted}
      class="w-1/2"
    />
    <DatePicker bind:day={editedDay} label="Day" />
  </div>
  <div class="flex items-centerr">
    <div class="w-1/2 px-2"></div>
    <div class="w-1/2 px-2"></div>
  </div>
  <Button
    type="submit"
    disabled={!valid}
    onclick={() => {
      if (!valid) return;
      onSubmit({
        day: editedDay,
        completed: editedCompleted,
        title: editedTitle,
      });
    }}
  >
    {#if data}
      Update
    {:else}
      Create
    {/if}
  </Button>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex flex-col gap-4 w-full;
  }
</style>
