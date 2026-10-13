<script lang="ts">
  import {
    createMutation,
    createQuery,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import { attachDraggable, attachDroppable } from "@thisux/sveltednd";

  import { invokeClient } from "../../client";
  import type { Placement, Tag, Todo } from "../../client/types";
  import {
    activeWorkspaceQueryOptions,
    invalidateTodosOf,
  } from "../../client/queries";
  import { createToggleTodoMutation } from "../../client/mutations";
  import EditableText from "#shared/ui/EditableText.svelte";
  import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";
  import TodoDescriptionMarker from "./TodoDescriptionMarker.svelte";
  import TodoDropdownMenu from "./TodoDropdownMenu.svelte";
  import TodoUpsertModal from "./TodoUpsertModal.svelte";
  import Button from "#shared/ui/Button.svelte";
  import IconEllipsisVertical from "#shared/ui/icons/IconEllipsisVertical.svelte";
  import IconClock from "#shared/ui/icons/IconClock.svelte";
  import TodoLinkChip from "../links/TodoLinkChip.svelte";
  import IconCheckCircle from "#shared/ui/icons/IconCheckCircle.svelte";
  import IconGripVertical from "#shared/ui/icons/IconGripVertical.svelte";
  import { doneSuggestion } from "../../utils/linkStatuses";
  import { taskProgress } from "../../utils/markdownEditing";
  import { formatEstimates } from "../../utils/estimates";

  type TodoLineItemProps = {
    item: Todo;
    tags: readonly Tag[];
    /**
     * Set to let it be reordered among the todos as completed as it, by dragging its handle (or from the keyboard, with
     * the handle focused): called with each one dropped on it.
     */
    onReorder?: (droppedId: Todo["id"], placement: Placement) => void;
  };
  const { item, tags, onReorder }: TodoLineItemProps = $props();

  /** What's dragged: as plain data, as it's serialized during a drag. Its title is announced from the keyboard. */
  type DraggedTodo = Pick<Todo, "id" | "title">;
  /** Completed todos and the others are reordered separately, as they're listed. */
  const dropGroup = $derived(item.completed ? "completed-todos" : "todos");

  let row = $state<HTMLLIElement>();
  /** While dragged or moved from the keyboard: not on a mere click of the handle, which also starts a drag. */
  let dragged = $state(false);
  /**
   * Whether the pointer is over it: tracked rather than read from `:hover`, which WebKit leaves on the row a drag started
   * from until the next click, as it never gets the mouse's release during a drag.
   */
  let hovered = $state(false);
  /** The row follows the pointer while its handle is dragged, rather than the handle alone. */
  const setRowDragImage = (event: DragEvent) => {
    if (!row || !event.dataTransfer) return;
    const { left, top } = row.getBoundingClientRect();
    event.dataTransfer.setDragImage(
      row,
      event.clientX - left,
      event.clientY - top,
    );
  };

  // Its GitHub links show their status through the workspace's connection, if any
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const connected = $derived(!!activeWorkspace.data?.githubLogin);

  // Resolved from the tags list, so they follow tag renames without refetching todos
  const itemTags = $derived(tags.filter((tag) => item.tagIds.includes(tag.id)));

  // Follows the server state, but is checked/unchecked right away on click.
  let completed = $derived(item.completed);
  const toggleCompleted = createToggleTodoMutation(
    () => item,
    () => (completed = item.completed),
  );
  /** Its merged pull request or completed issue, if any, which suggests marking it as done. */
  const doneBecause = $derived(
    connected && activeWorkspace.data?.linksSuggestDone
      ? doneSuggestion({ completed, links: item.links })
      : undefined,
  );

  const queryClient = useQueryClient();

  // svelte-ignore state_referenced_locally: edited locally, then saved
  let editedTitle = $state(item.title);
  const updateTitle = createMutation(() => ({
    mutationFn: (title: string) =>
      invokeClient({
        name: "update_todo",
        args: { params: { id: item.id, title } },
      }),
    onSuccess: () => {
      return invalidateTodosOf(queryClient, item.day);
    },
    onError: () => {
      editedTitle = item.title;
    },
  }));
  const saveTitle = (title: string) => {
    editedTitle = title;
    // Also called when confirming the unchanged title
    if (title !== item.title) updateTitle.mutate(title);
  };

  let openedDropdownMenu = $state(false);
  let menuButton = $state<HTMLButtonElement>();

  let showEditModal = $state(false);
  /** Whether the edit modal was opened to edit the description: its editor is then focused. */
  let editingDescription = $state(false);
  const openEditModal = (description: boolean) => {
    editingDescription = description;
    showEditModal = true;
  };

  /** Its description's tasks, counted next to its tags. */
  const tasks = $derived(taskProgress(item.description ?? ""));
</script>

<li
  bind:this={row}
  class={[
    "todo-item",
    item.completed && "completed",
    dragged && "dragged",
    hovered && "hovered",
  ]}
  onpointerenter={() => (hovered = true)}
  onpointerleave={() => (hovered = false)}
  {@attach attachDroppable<DraggedTodo>(() => ({
    container: item.id,
    containerGroup: dropGroup,
    disabled: !onReorder,
    attributes: { dragOverClass: "drop-target" },
    callbacks: {
      onDrop: ({ draggedItem, dropPosition }) => {
        if (dropPosition) onReorder?.(draggedItem.id, dropPosition);
      },
    },
  }))}
>
  {#if onReorder}
    <!-- The only part to drag the todo by: dragging the whole row would get in the way of selecting text in its
         title's field. Focusable, to move it with the keyboard (Space, arrows, Space). -->
    <span
      class="drag-handle"
      role="button"
      tabindex="0"
      aria-label={`Reorder “${item.title}”`}
      title="Drag to reorder"
      ondragstart={setRowDragImage}
      {@attach attachDraggable<DraggedTodo>(() => ({
        container: item.id,
        containerGroup: dropGroup,
        dragData: { id: item.id, title: item.title },
        keyboard: true,
        attributes: { draggingClass: "grabbed" },
        callbacks: {
          // Once its drag image is taken, so that it isn't dimmed too
          onDragStart: ({ dragInput }) => {
            if (dragInput !== "pointer") setTimeout(() => (dragged = true));
          },
          onDragEnd: () => (dragged = false),
        },
      }))}
    >
      <IconGripVertical class="size-5" />
    </span>
  {/if}
  <div class="flex items-center">
    <FieldCheckbox
      label="Completed?"
      hideLabel
      bind:checked={completed}
      onchange={(event) => toggleCompleted.mutate(event.currentTarget.checked)}
    />
    <div class="flex flex-col gap-1">
      <div class="flex items-center gap-1">
        <EditableText
          as="h3"
          bind:value={() => editedTitle, saveTitle}
          label={`Title of todo "${item.title}"`}
          class="text-sm font-medium text-white wrap-break-word"
        />
        {#if item.description}
          <TodoDescriptionMarker todo={item} onEdit={() => openEditModal(true)} />
        {/if}
      </div>
      {#if itemTags.length > 0 || tasks.total > 0 || item.links.length > 0}
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
          {#if tasks.total > 0}
            <span
              class={["tasks", tasks.done === tasks.total && "done"]}
              title={`${tasks.done} of ${tasks.total} tasks done`}
            >
              <IconCheckCircle />
              <span aria-hidden="true">{tasks.done}/{tasks.total}</span>
              <span class="sr-only">{tasks.done} of {tasks.total} tasks done</span>
            </span>
          {/if}
          {#if item.links.length > 0}
            {#if itemTags.length > 0 || tasks.total > 0}
              <span class="links-divider" aria-hidden="true"></span>
            {/if}
            <ul class="links" aria-label="Links">
              {#each item.links as link (link.id)}
                <li class="flex">
                  <TodoLinkChip {link} todo={item} {connected} />
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </div>
  </div>
  <div class="shrink-0">
    <div class="flex items-center gap-2">
      {#if doneBecause}
        <!-- Only suggested: a todo is never completed on its own -->
        <Button
          style="plain"
          size="xs"
          class="suggest-done"
          title={`${doneBecause.link.label} was ${doneBecause.state}`}
          disabled={toggleCompleted.isPending}
          onclick={() => {
            completed = true;
            toggleCompleted.mutate(true);
          }}
        >
          <IconCheckCircle />
          {doneBecause.state === "merged" ? "Merged" : "Completed"} · mark as done
        </Button>
      {/if}
      {#if item.estimate}
        <div class="flex items-center gap-1 text-white">
          {#if item.estimate.unit === "minutes"}
            <IconClock />
          {/if}
          <span class="text-sm">{formatEstimates([item.estimate])}</span>
        </div>
      {/if}
      <Button
        style="plain"
        bind:ref={menuButton}
        aria-haspopup="menu"
        aria-expanded={openedDropdownMenu}
        onclick={() => (openedDropdownMenu = true)}
      >
        <IconEllipsisVertical />
      </Button>
      <TodoDropdownMenu
        bind:open={openedDropdownMenu}
        anchor={menuButton}
        {item}
        {tags}
        onEdit={() => openEditModal(false)}
      />
    </div>
  </div>
</li>
<!-- Only while shown. Out of the line, which handles dragging. -->
{#if showEditModal}
  <TodoUpsertModal
    bind:show={showEditModal}
    existing={item}
    focusDescription={editingDescription}
  />
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  /* A `Button`, restyled through its line as it's another component's element: more specific than its own rules */
  .todo-item :global(.suggest-done) {
    @apply inline-flex h-6.5 shrink-0 items-center gap-1.5 rounded-md border-0 px-2.5 py-0;
    @apply bg-purple-400/16 text-xs/4 font-semibold text-purple-200 ring-1 ring-purple-400/35 ring-inset;
    @apply transition-colors disabled:cursor-default disabled:opacity-60 focus-visible:outline-offset-0;

    /* Highlighted when hovered, even disabled, but not when pressed */
    &:enabled:active {
      @apply bg-purple-400/16;
    }
    &:enabled:hover,
    &:disabled:hover {
      @apply hover:bg-purple-400/25;
    }

    > :global(svg) {
      @apply size-3.5;
    }
  }

  .todo-item {
    /* Room on the left for the drag handle, kept without it (e.g. while filtered) so that rows never shift */
    @apply relative flex items-center justify-between gap-1 py-3 pr-4 pl-7.5 rounded-lg;
    @apply border border-slate-600 bg-slate-800;
    @apply transition-colors;
    /* Rather than `:hover`: see `hovered` */
    &.hovered {
      @apply border-slate-400 bg-slate-600;
    }
    &.completed {
      @apply border-gray-700 bg-gray-900;
      &.hovered {
        @apply bg-gray-700;
      }
    }
    &.dragged {
      @apply opacity-40;
    }
  }

  /* In the row's padding, before the checkbox: shown with the row's hover, or when focused. Centered on the checkbox
     rather than the row: `FieldCheckbox` keeps its box 2px lower, on its label's first line, even with the label hidden. */
  .drag-handle {
    @apply absolute top-[calc(50%+2px)] left-1.5 flex h-8 w-5 -translate-y-1/2 cursor-grab items-center justify-center rounded-md;
    @apply text-slate-500 opacity-0 transition-opacity hover:text-slate-200;
    @apply focus:not-focus-visible:outline-hidden focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-blue-500;
    .todo-item.hovered &,
    &:global(.grabbed) {
      @apply opacity-100;
    }
  }

  /* Where a dragged todo would go: a line centered in the gap above or below the row (the list's `gap-1.5`, past the
     row's border). The library draws it; only its look is the app's. */
  .todo-item:global(.drop-before)::before,
  .todo-item:global(.drop-after)::after {
    @apply h-0.5 rounded-full bg-pink-400;
  }
  .todo-item:global(.drop-before)::before {
    @apply top-[-5px];
  }
  .todo-item:global(.drop-after)::after {
    @apply bottom-[-5px];
  }
  /* Not around the dragged todo itself, where dropping it changes nothing */
  .todo-item.dragged:global(.drop-before)::before,
  .todo-item.dragged:global(.drop-after)::after {
    @apply hidden;
  }

  /* Between the tags and the links, which are told apart at a glance */
  .links-divider {
    @apply -mx-1 h-3.5 w-px shrink-0 bg-white/10;
  }

  .links {
    @apply flex shrink-0 items-center gap-1.5;
  }

  /* Its description's tasks done, as a chip like its links, in the notes' accent; muted once all done */
  .tasks {
    @apply inline-flex h-5 shrink-0 items-center gap-1 rounded-md bg-white/5 px-1.5 ring-1 ring-white/10 ring-inset;
    @apply text-xs/4 font-medium text-slate-300 tabular-nums;
    > :global(svg) {
      @apply size-3 text-pink-400;
    }
    &.done {
      @apply text-slate-400;
      > :global(svg) {
        @apply text-slate-400;
      }
    }
  }
</style>
