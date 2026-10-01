<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import { invokeClient } from "../../client";
  import { todoKeys } from "../../client/queries";
  import type { Day } from "../../client/types";
  import FieldText from "../inputs/FieldText.svelte";

  type TodoCreateSectionProps = {
    day: Day;
  };
  const { day }: TodoCreateSectionProps = $props();

  const queryClient = useQueryClient();

  let title = $state("");
  const createTodo = createMutation(() => ({
    mutationFn: (title: string) =>
      invokeClient({ name: "create_todo", args: { day, title } }),
    onSuccess: () => {
      title = "";
      // Returned so the mutation stays pending until the list is refetched.
      return queryClient.invalidateQueries({ queryKey: todoKeys.day(day) });
    },
  }));

  const onsubmit = (event: SubmitEvent) => {
    event.preventDefault();
    if (createTodo.isPending || !title.trim()) {
      return;
    }
    createTodo.mutate(title);
  };
</script>

<form class="container" {onsubmit}>
  <FieldText
    label="New todo"
    hideLabel
    placeholder="Add a new todo item"
    autocomplete="off"
    bind:value={title}
  />
  {#if createTodo.isError}
    <p class="error" role="alert">{String(createTodo.error)}</p>
  {/if}
</form>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full flex flex-col gap-1 p-2;
  }

  .error {
    @apply text-sm text-red-700;
  }
</style>
