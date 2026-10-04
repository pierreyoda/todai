<script lang="ts">
  import {
    Dropdown,
    DropdownMenu,
    DropdownSubmenu,
    DropdownItem,
    DropdownDivider,
  } from "../common/dropdown";
  import type { Todo, Tag } from "../../client/types";
  import TodoLineItemTagToggle from "./TodoLineItemTagToggle.svelte";
  import TagUpsertModal from "../tags/TagUpsertModal.svelte";
  import { createDeleteTodoMutation } from "../../client/mutations";

  type TodoDropdownMenuProps = {
    anchor?: HTMLElement;
    /** Bindable value. */
    open?: boolean;
    item: Todo;
    tags: readonly Tag[];
    inCalendar?: boolean;
  };
  let {
    anchor,
    open = $bindable(false),
    item,
    tags,
    inCalendar = false,
  }: TodoDropdownMenuProps = $props();

  let showTagCreationModal = $state(false);

  // Created during initialization, as it reads the query client from the context
  const deleteTodo = createDeleteTodoMutation(() => item);
  let confirmItemDeletion = $state(false);
  $effect(() => {
    if (!open) confirmItemDeletion = false;
  });
</script>

<Dropdown bind:open {anchor}>
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
    <DropdownItem
      onclick={() => {
        if (confirmItemDeletion) deleteTodo.mutate();
        else confirmItemDeletion = true;
      }}
    >
      <span class="text-red-400">
        {#if confirmItemDeletion}
          Confirm
        {:else}
          Delete
        {/if}
      </span>
    </DropdownItem>
  </DropdownMenu>
</Dropdown>
<TagUpsertModal bind:show={showTagCreationModal} />
