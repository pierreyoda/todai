<script lang="ts" module>
  export type TodoFormData = Pick<Todo, "day" | "title" | "completed">;
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  import type { Todo } from "../model/types";
  import { dateToTodaiDate, isTodaiDate } from "#core/time";
  import Button from "#shared/ui/Button.svelte";
  import { DatePicker } from "#core/time";
  import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";
  import FieldText from "#shared/ui/FieldText.svelte";

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
    /**
     * Under the title and `fields`, taking the height left: e.g. the todo's description. Lays the form out in two
     * columns, its other fields on the side: the form must then be in a flex column of bounded height.
     */
    main?: Snippet;
    /**
     * Fields saved on their own rather than submitted, e.g. the todo's links: under the title in two columns, before the
     * submit button otherwise.
     */
    fields?: Snippet;
    /** Next to the submit button, in two columns: e.g. what it saves. */
    hint?: string;
  };
  const { data, onSubmit, main, fields, hint }: TodoFormProps = $props();

  let editedDay = $state(data?.day ?? dateToTodaiDate(new Date()));
  let editedTitle = $state(data?.title ?? "");
  let editedCompleted = $state(data?.completed ?? false);
  const valid = $derived(
    isTodaiDate(editedDay) && editedTitle.trim().length > 0,
  );

  const submit = () => {
    if (!valid) return;
    onSubmit({
      day: editedDay,
      completed: editedCompleted,
      title: editedTitle,
    });
  };
</script>

{#snippet submitButton()}
  <Button type="submit" disabled={!valid} onclick={submit}>
    {#if data}
      Update
    {:else}
      Create
    {/if}
  </Button>
{/snippet}

{#if main}
  <div class="columns">
    <div class="main">
      <FieldText bind:value={editedTitle} label="Title" />
      {@render fields?.()}
      {@render main()}
    </div>
    <div class="side">
      <!-- Raised onto the line of the title's label: its own line is 4px taller -->
      <FieldCheckbox
        label="Completed"
        bind:checked={editedCompleted}
        class="-mt-0.5"
      />
      <div class="day">
        <span class="label">Day</span>
        <DatePicker bind:day={editedDay} label="Day" class="self-start" />
      </div>
    </div>
    <div class="footer">
      {#if hint}
        <p class="hint">{hint}</p>
      {/if}
      {@render submitButton()}
    </div>
  </div>
{:else}
  <div class="container">
    <FieldText bind:value={editedTitle} label="Title" />
    <div class="flex items-center">
      <FieldCheckbox
        label="Completed"
        bind:checked={editedCompleted}
        class="w-1/2"
      />
      <DatePicker bind:day={editedDay} label="Day" class="ml-auto" />
    </div>
    <div class="flex items-center">
      <div class="w-1/2 px-2"></div>
      <div class="w-1/2 px-2"></div>
    </div>
    {@render fields?.()}
    {@render submitButton()}
  </div>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex flex-col gap-4 w-full;
  }

  /* What the todo is on the left, taking the height left; its properties on the right */
  .columns {
    @apply grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_15.5rem] grid-rows-[minmax(0,1fr)_auto] gap-x-6 gap-y-4;
  }

  .main {
    @apply flex min-h-0 flex-col gap-4;
  }

  .side {
    @apply flex flex-col gap-5 border-l border-white/5 pl-6;
  }

  .day {
    @apply flex flex-col gap-1.5;
  }

  /* As the fields' labels */
  .label {
    @apply text-sm/5 font-medium text-slate-300;
  }

  .footer {
    @apply col-span-full mt-2 flex items-center justify-end gap-3;
  }

  .hint {
    @apply mr-auto text-xs/5 text-slate-400;
  }
</style>
