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
  import {
    createDeleteTodoMutation,
    createRemoveTodoLinkMutation,
    createSetTodoEstimateMutation,
  } from "../../client/mutations";
  import {
    TODO_ESTIMATE_MINUTES_PRESETS,
    TODO_ESTIMATE_POINTS_PRESET,
  } from "../../constants";
  import { formatEstimates } from "../../utils";
  import IconClock from "../common/icons/IconClock.svelte";
  import IconLink from "../common/icons/IconLink.svelte";
  import IconTag from "../common/icons/IconTag.svelte";
  import IconTrash from "../common/icons/IconTrash.svelte";
  import IconXMark from "../common/icons/IconXMark.svelte";
  import AddLinkModal from "../links/AddLinkModal.svelte";
  import LinkKindIcon from "../links/LinkKindIcon.svelte";

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
  let showAddLinkModal = $state(false);

  const setTodoEstimate = createSetTodoEstimateMutation(() => item);
  const removeLink = createRemoveTodoLinkMutation(() => item);

  // Created during initialization, as it reads the query client from the context
  const deleteTodo = createDeleteTodoMutation(() => item);
  let confirmItemDeletion = $state(false);
  $effect(() => {
    if (!open) confirmItemDeletion = false;
  });
</script>

<Dropdown bind:open {anchor}>
  <DropdownMenu placement="bottom-end">
    {@const estimateLabel = item.estimate
      ? `Estimate (${formatEstimates([item.estimate])})`
      : "Estimate"}
    <DropdownSubmenu label={estimateLabel} keepOpenOnClick={false}>
      {#snippet icon()}
        <IconClock />
      {/snippet}
      <DropdownSubmenu label="Time">
        {#each TODO_ESTIMATE_MINUTES_PRESETS as { value } (value)}
          <DropdownItem
            onclick={() =>
              setTodoEstimate.mutate({
                unit: "minutes",
                value,
              })}
          >
            {formatEstimates([{ unit: "minutes", value }])}
          </DropdownItem>
        {/each}
      </DropdownSubmenu>
      <DropdownSubmenu label="Story points">
        {#each TODO_ESTIMATE_POINTS_PRESET as { value } (value)}
          <DropdownItem
            onclick={() =>
              setTodoEstimate.mutate({
                unit: "points",
                value,
              })}
          >
            {formatEstimates([{ unit: "points", value }])}
          </DropdownItem>
        {/each}
      </DropdownSubmenu>
      {#if item.estimate}
        <DropdownDivider />
        <DropdownItem onclick={() => setTodoEstimate.mutate(null)}>
          Clear
        </DropdownItem>
      {/if}
    </DropdownSubmenu>
    <DropdownSubmenu label="Toggle tags">
      {#snippet icon()}
        <IconTag />
      {/snippet}
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
      <!-- Opens a modal: the menu must close first, see `keepOpen` -->
      <DropdownItem
        keepOpen={false}
        onclick={() => (showTagCreationModal = true)}
      >
        + Add a new tag
      </DropdownItem>
    </DropdownSubmenu>
    <!-- Opens a modal: the menu must close first, see `keepOpen` -->
    <DropdownItem keepOpen={false} onclick={() => (showAddLinkModal = true)}>
      {#snippet icon()}
        <IconLink />
      {/snippet}
      Add a link…
    </DropdownItem>
    {#if item.links.length > 0}
      <DropdownSubmenu label="Remove a link" keepOpenOnClick={false}>
        {#snippet icon()}
          <IconXMark />
        {/snippet}
        {#each item.links as link (link.id)}
          <DropdownItem onclick={() => removeLink.mutate(link)}>
            {#snippet icon()}
              <LinkKindIcon kind={link.kind} />
            {/snippet}
            {link.label}
          </DropdownItem>
        {/each}
      </DropdownSubmenu>
    {/if}
    <DropdownDivider />
    <DropdownItem
      onclick={() => {
        if (confirmItemDeletion) deleteTodo.mutate();
        else confirmItemDeletion = true;
      }}
    >
      {#snippet icon()}
        <IconTrash class="text-red-400" />
      {/snippet}
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
<AddLinkModal bind:show={showAddLinkModal} todo={item} />
