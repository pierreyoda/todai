<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import type { Tag, Todo } from "../../client/types";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";
  import { invokeClient } from "../../client";
  import { todoKeys } from "../../client/queries";
  import {
    Dropdown,
    DropdownButton,
    DropdownDivider,
    DropdownItem,
    DropdownMenu,
    DropdownSubmenu,
  } from "../common/dropdown";
  import EllipsisVertical from "../common/icons/EllipsisVertical.svelte";
  import TodoLineItemTagToggle from "./TodoLineItemTagToggle.svelte";
  import TagUpsertModal from "../tags/TagUpsertModal.svelte";

  type TodoLineItemProps = {
    item: Todo;
    tags: readonly Tag[];
  };
  const { item, tags }: TodoLineItemProps = $props();

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

  let showTagCreationModal = $state(false);
</script>

<li class={["todo-item", item.completed && "completed"]}>
  <div class="flex items-center">
    <FieldCheckbox
      label="Completed?"
      hideLabel
      bind:checked={completed}
      onchange={(event) => toggleCompleted.mutate(event.currentTarget.checked)}
    />
    <div class="flex flex-col gap-1">
      <h3 class="title">{item.title}</h3>
      {#if item.tags.length > 0}
        <div class="flex items-center gap-4 overflow-x-auto">
          {#each item.tags as itemTag (itemTag.id)}
            <div class="flex items-center gap-1">
              <div
                class={["rounded-full w-2 h-2"]}
                style:background-color={itemTag.color}
              ></div>
              <span class="text-white text-xs">{itemTag.name}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
  <Dropdown>
    <DropdownButton style="plain">
      <EllipsisVertical />
    </DropdownButton>
    <DropdownMenu placement="bottom-end">
      <DropdownSubmenu label="Toggle tags">
        {#each tags as tag (tag.id)}
          <DropdownItem onclick={() => {}}>
            <TodoLineItemTagToggle
              todoId={item.id}
              todoTagsIds={item.tags.map(({ id }) => id)}
              {tag}
            />
          </DropdownItem>
        {/each}
        <DropdownDivider />
        <DropdownItem onclick={() => (showTagCreationModal = true)}>
          + Add a new tag
        </DropdownItem>
      </DropdownSubmenu>
      <DropdownDivider />
      <DropdownItem onclick={console.log}>
        <span class="text-red-400">Delete</span>
      </DropdownItem>
    </DropdownMenu>
  </Dropdown>
</li>
<TagUpsertModal bind:show={showTagCreationModal} />

<style lang="postcss">
  @reference "tailwindcss";

  .todo-item {
    @apply flex items-center justify-between gap-1 px-4 py-3 rounded-lg;
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
