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
    createMoveTodoMutation,
    createRemoveTodoLinkMutation,
    createSetTodoEstimateMutation,
  } from "../../client/mutations";
  import {
    TODO_ESTIMATE_MINUTES_PRESETS,
    TODO_ESTIMATE_POINTS_PRESET,
  } from "../../constants";
  import { formatEstimates } from "../../utils";
  import { addDays, dateToTodaiDate, formatDay } from "../../utils/dates";
  import DatePickerModal from "../common/DatePickerModal.svelte";
  import IconArrowRight from "../common/icons/IconArrowRight.svelte";
  import IconCalendar from "../common/icons/IconCalendar.svelte";
  import IconClock from "../common/icons/IconClock.svelte";
  import IconPencilSquare from "../common/icons/IconPencilSquare.svelte";
  import IconSun from "../common/icons/IconSun.svelte";
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
    /** Opens its edit modal, which its line has: also opened otherwise, e.g. from its description. */
    onEdit: () => void;
  };
  let {
    anchor,
    open = $bindable(false),
    item,
    tags,
    inCalendar = false,
    onEdit,
  }: TodoDropdownMenuProps = $props();

  let showTagCreationModal = $state(false);
  let showAddLinkModal = $state(false);
  let showDatePicker = $state(false);

  /** Updated when opened, so that a menu opened after midnight offers the right days. */
  let today = $state(dateToTodaiDate(new Date()));
  $effect(() => {
    if (open) today = dateToTodaiDate(new Date());
  });
  const tomorrow = $derived(addDays(today, 1));
  /** Today and the next few days, but its own. */
  const moveTargets = $derived(
    [0, 1, 2, 3, 4]
      .map((count) => addDays(today, count))
      .filter((day) => day !== item.day),
  );
  // Leaves the list it's moved from: tells where it went, with an undo
  const moveTodo = createMoveTodoMutation(() => item, { undoable: true });

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
    <!-- Opens a modal: the menu must close first, see `keepOpen` -->
    <DropdownItem keepOpen={false} onclick={onEdit}>
      {#snippet icon()}
        <IconPencilSquare />
      {/snippet}
      Edit…
    </DropdownItem>
    <DropdownSubmenu label="Move to" keepOpenOnClick={false}>
      {#snippet icon()}
        <IconArrowRight />
      {/snippet}
      {#each moveTargets as day (day)}
        <DropdownItem
          disabled={moveTodo.isPending}
          onclick={() => moveTodo.mutate(day)}
        >
          {#snippet icon()}
            {#if day === today}
              <IconSun />
            {/if}
          {/snippet}
          {#if day === today}
            Today
          {:else if day === tomorrow}
            Tomorrow <span class="hint">{formatDay(day)}</span>
          {:else}
            {formatDay(day)}
          {/if}
        </DropdownItem>
      {/each}
      <DropdownDivider />
      <!-- Opens a modal: the menu must close first, see `keepOpen` -->
      <DropdownItem keepOpen={false} onclick={() => (showDatePicker = true)}>
        {#snippet icon()}
          <IconCalendar />
        {/snippet}
        Pick a date…
      </DropdownItem>
    </DropdownSubmenu>
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
<!-- Only while shown: the calendar has a menu per todo -->
{#if showDatePicker}
  <DatePickerModal
    bind:show={showDatePicker}
    day={item.day}
    label="Move to"
    onPick={(day) => {
      if (day !== item.day) moveTodo.mutate(day);
    }}
  />
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  /* After the day's name, as the side panel's hint for Today */
  .hint {
    @apply ml-1.5 text-white/50;
  }
</style>
