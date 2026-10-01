<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import type { Todo } from "../../client/types";
  import FieldCheckbox from "../inputs/FieldCheckbox.svelte";
  import { invokeClient } from "../../client";
  import { todoKeys } from "../../client/queries";

  type TodoLineItemProps = {
    item: Todo;
  };
  const { item }: TodoLineItemProps = $props();

  // Follows the server state, but is checked/unchecked right away on click.
  let completed = $derived(item.completed);
  const queryClient = useQueryClient();
  const toggleCompleted = createMutation(() => ({
    mutationFn: (completed: boolean) =>
      invokeClient({
        name: "toggle_todo",
        args: { id: item.id, completed },
      }),
    onSuccess: () => {
      // Returned so the mutation stays pending until the list is refetched.
      return queryClient.invalidateQueries({
        queryKey: todoKeys.day(item.day),
      });
    },
    onError: () => {
      completed = item.completed;
    },
  }));
</script>

<li class={["todo-item", item.completed && "completed"]}>
  <FieldCheckbox
    label="Completed?"
    hideLabel
    bind:checked={completed}
    onchange={(event) => toggleCompleted.mutate(event.currentTarget.checked)}
  />
  <h3 class="title">{item.title}</h3>
</li>

<style lang="postcss">
  @reference "tailwindcss";

  .todo-item {
    @apply flex gap-1 px-4 py-3 rounded-lg;
    @apply border border-slate-600 bg-slate-800;
    @apply transition-colors hover:border-slate-400 hover:bg-slate-600;
    &.completed {
      @apply border-gray-700 bg-gray-900 hover:bg-gray-700;
    }

    .title {
      @apply text-sm font-medium text-white wrap-break-word;
    }
  }
</style>
