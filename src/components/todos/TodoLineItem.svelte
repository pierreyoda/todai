<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import type { Tag, Todo } from "../../client/types";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";
  import EditableText from "../common/EditableText.svelte";
  import { invokeClient } from "../../client";
  import { invalidateTodosOf } from "../../client/queries";
  import { createToggleTodoMutation } from "../../client/mutations";
  import { Debounced } from "../../utils/debounced.svelte";
  import {
    Dropdown,
    DropdownButton,
    DropdownDivider,
    DropdownItem,
    DropdownMenu,
    DropdownSubmenu,
  } from "../common/dropdown";
  import EllipsisVertical from "../common/icons/IconEllipsisVertical.svelte";
  import TodoLineItemTagToggle from "./TodoLineItemTagToggle.svelte";
  import TagUpsertModal from "../tags/TagUpsertModal.svelte";

  type TodoLineItemProps = {
    item: Todo;
    tags: readonly Tag[];
  };
  const { item, tags }: TodoLineItemProps = $props();

  // Resolved from the tags list, so they follow tag renames without refetching todos
  const itemTags = $derived(tags.filter((tag) => item.tagIds.includes(tag.id)));

  // Follows the server state, but is checked/unchecked right away on click.
  let completed = $derived(item.completed);
  const toggleCompleted = createToggleTodoMutation(
    () => item,
    () => (completed = item.completed),
  );

  const queryClient = useQueryClient();

  // svelte-ignore state_referenced_locally: edited locally, then saved
  let editedTitle = $state(item.title);
  const updateTitle = createMutation(() => ({
    mutationFn: (title: string) =>
      invokeClient({
        name: "update_todo",
        args: { id: item.id, title },
      }),
    onSuccess: () => {
      return invalidateTodosOf(queryClient, item.day);
    },
    onError: () => {
      editedTitle = item.title;
    },
  }));
  const debouncedEditedTitle = new Debounced(() => editedTitle, 500);
  $effect(() => {
    const title = debouncedEditedTitle.current;
    // Also runs on mount, with the unchanged title
    if (title === item.title) return;
    // `mutate` reads the mutation's state, which it then updates: untracked so the effect doesn't loop
    untrack(() => updateTitle.mutate(title));
  });

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
      <EditableText
        as="h3"
        bind:value={editedTitle}
        label={`Title of todo "${item.title}"`}
        class="text-sm font-medium text-white wrap-break-word"
      />
      {#if itemTags.length > 0}
        <div class="flex items-center gap-4 overflow-x-auto">
          {#each itemTags as itemTag (itemTag.id)}
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
              todoDay={item.day}
              todoTagsIds={item.tagIds}
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
  }
</style>
