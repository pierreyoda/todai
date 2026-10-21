<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { attachDraggable, attachDroppable } from "@thisux/sveltednd";
import { taskProgress } from "#core/markdown";
import type { Tag } from "#domains/tags";
import { activeWorkspaceQueryOptions, isConnected } from "#domains/workspaces";
import Button from "#shared/ui/Button.svelte";
import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";
import IconCheckCircle from "#shared/ui/icons/IconCheckCircle.svelte";
import IconClock from "#shared/ui/icons/IconClock.svelte";
import IconEllipsisVertical from "#shared/ui/icons/IconEllipsisVertical.svelte";
import IconPlay from "#shared/ui/icons/IconPlay.svelte";
import { formatDateTime } from "#shared/utils/timestamps.ts";
import { createToggleTodoMutation } from "../../api/mutations";
import { linkProvider } from "../../links/model/links";
import type { TodoLink } from "../../links/model/types";
import TodoLinkChip from "../../links/ui/TodoLinkChip.svelte";
import { formatEstimates } from "../../model/estimates";
import type { Placement, Todo } from "../../model/types";
import TodoBadge from "../../ui/TodoBadge.svelte";
import TodoDescriptionMarker from "../../ui/TodoDescriptionMarker.svelte";
import TodoDropdownMenu from "../../ui/TodoDropdownMenu.svelte";
import TodoUpsertModal from "../../ui/TodoUpsertModal.svelte";
import { type BoardColumn, formatStarted } from "../model/board";
import { BOARD_DROP_GROUP, type DraggedTodo } from "./dragging";

type BoardCardProps = {
  item: Todo;
  tags: readonly Tag[];
  /** Its column: in Doing, it also tells since when it was started, and whether it's urgent. */
  column: BoardColumn;
  /** Set to let todos be dropped on it, right before or after it: called with each one dropped. */
  onDrop?: (droppedId: Todo["id"], placement: Placement) => void;
};
const { item, tags, column, onDrop }: BoardCardProps = $props();

/** While dragged or moved from the keyboard: dimmed, once its drag image is taken. */
let dragged = $state(false);
/**
 * Whether the pointer is over it: tracked rather than read from `:hover`, which WebKit leaves on the card a drag started
 * from until the next click, as it never gets the mouse's release during a drag.
 */
let hovered = $state(false);

// Its links show their status through the workspace's connection to their provider, if any
const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
const connected = (link: TodoLink) => {
  const provider = linkProvider(link);
  return !!provider && isConnected(activeWorkspace.data, provider);
};

// Resolved from the tags list, so they follow tag renames without refetching todos
const itemTags = $derived(tags.filter((tag) => item.tagIds.includes(tag.id)));

// Follows the server state, but is checked/unchecked right away on click: it then changes column once refetched
let completed = $derived(item.completed);
const toggleCompleted = createToggleTodoMutation(() => item, {
  onError: () => (completed = item.completed),
});

/** In Doing, since when: its time if today, else its day. */
const started = $derived(
  column === "doing" && item.startedAt
    ? formatStarted(item.startedAt, new Date())
    : undefined,
);
/** Its description's tasks, counted next to its tags. */
const tasks = $derived(taskProgress(item.description ?? ""));
const hasMeta = $derived(
  (column === "doing" && (item.urgent || !!started)) ||
    itemTags.length > 0 ||
    tasks.total > 0 ||
    item.links.length > 0 ||
    !!item.estimate,
);

let openedDropdownMenu = $state(false);
let menuButton = $state<HTMLButtonElement>();

let showEditModal = $state(false);
/** Whether the edit modal was opened to edit the description: its editor is then focused. */
let editingDescription = $state(false);
const openEditModal = (description: boolean) => {
  editingDescription = description;
  showEditModal = true;
};
</script>

<!-- Dragged by any part but its controls (checkbox, buttons, links), or from the keyboard once focused (Space, arrows,
     Space). Arrows go through the cards column by column, hence `horizontal`. -->
<li
  class={[
    "card",
    item.completed && "completed",
    dragged && "dragged",
    hovered && "hovered",
  ]}
  onpointerenter={() => (hovered = true)}
  onpointerleave={() => (hovered = false)}
  {@attach attachDroppable<DraggedTodo>(() => ({
    container: item.id,
    containerGroup: BOARD_DROP_GROUP,
    disabled: !onDrop,
    attributes: { dragOverClass: "drop-target" },
    callbacks: {
      onDrop: ({ draggedItem, dropPosition }) => {
        if (dropPosition) onDrop?.(draggedItem.id, dropPosition);
      },
    },
  }))}
  {@attach attachDraggable<DraggedTodo>(() => ({
    container: item.id,
    containerGroup: BOARD_DROP_GROUP,
    dragData: { id: item.id, title: item.title },
    keyboard: true,
    direction: "horizontal",
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
  <div class="top">
    <FieldCheckbox
      label={`Completed: ${item.title}`}
      hideLabel
      bind:checked={completed}
      onchange={(event) => toggleCompleted.mutate(event.currentTarget.checked)}
    />
    <h3 class="title">{item.title}</h3>
    {#if item.description}
      <TodoDescriptionMarker todo={item} onEdit={() => openEditModal(true)} />
    {/if}
    <Button
      style="plain"
      size="xs"
      bind:ref={menuButton}
      aria-label={`Actions for “${item.title}”`}
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
  {#if hasMeta}
    <div class="meta">
      <!-- The other columns tell it: Urgent is all urgent, and Done is done with -->
      {#if column === "doing" && item.urgent}
        <TodoBadge kind="urgent" />
      {/if}
      {#if started && item.startedAt}
        <span class="since" title={`Started: ${formatDateTime(item.startedAt)}`}>
          <IconPlay />
          Since {started}
        </span>
      {/if}
      {#each itemTags as itemTag (itemTag.id)}
        <span class="tag">
          <span class="tag-color" style:background-color={itemTag.color}></span>
          {itemTag.name}
        </span>
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
        <ul class="links" aria-label="Links">
          {#each item.links as link (link.id)}
            <li class="flex">
              <TodoLinkChip {link} todo={item} connected={connected(link)} />
            </li>
          {/each}
        </ul>
      {/if}
      <!-- Last, rather than next to the title, which then has the card's width -->
      {#if item.estimate}
        <span class="estimate">
          {#if item.estimate.unit === "minutes"}
            <IconClock />
          {/if}
          {formatEstimates([item.estimate])}
        </span>
      {/if}
    </div>
  {/if}
</li>
<!-- Only while shown. Out of the card, which handles dragging. -->
{#if showEditModal}
  <TodoUpsertModal
    bind:show={showEditModal}
    existing={item}
    focusDescription={editingDescription}
  />
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .card {
    /* Layout: its checkbox lines up with the title's first line */
    @apply relative flex flex-col gap-1.5 rounded-lg py-2.5 pr-1.5 pl-3;
    @apply cursor-grab border border-slate-600 bg-slate-800 transition-colors;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
    /* Rather than `:hover`: see `hovered` */
    &.hovered {
      @apply border-slate-400 bg-slate-700;
    }
    &.completed {
      @apply border-gray-700 bg-gray-900;
      &.hovered {
        @apply bg-gray-800;
      }
      .title {
        @apply text-slate-400;
      }
    }
    &.dragged {
      @apply opacity-40;
    }
  }

  .top {
    @apply flex items-start gap-2;
  }

  .title {
    @apply min-w-0 flex-1 text-sm/6 font-medium wrap-break-word text-white;
  }

  /* At the end of its line */
  .estimate {
    @apply ml-auto flex shrink-0 items-center gap-1 text-xs/4 text-slate-300 tabular-nums;
    > :global(svg) {
      @apply size-3.5 text-slate-400;
    }
  }

  /* Under the title, past the checkbox */
  .meta {
    @apply flex flex-wrap items-center gap-x-3 gap-y-1.5 pr-1.5 pl-7.5;
  }

  .since {
    @apply inline-flex shrink-0 items-center gap-1 text-xs/4 font-medium text-sky-300 tabular-nums;
    > :global(svg) {
      @apply size-3.5;
    }
  }

  .tag {
    @apply inline-flex items-center gap-1 text-xs/4 text-white;
  }

  .tag-color {
    @apply size-2 shrink-0 rounded-full;
  }

  .links {
    @apply flex flex-wrap items-center gap-1.5;
  }

  /* Its description's tasks done, as the list shows them */
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

  /* Where a dragged todo would go: a line centered in the gap above or below the card (the column's `gap-1.5`, past
     the card's border). The library draws it; only its look is the app's. */
  .card:global(.drop-before)::before,
  .card:global(.drop-after)::after {
    @apply h-0.5 rounded-full bg-pink-400;
  }
  .card:global(.drop-before)::before {
    @apply -top-1.25;
  }
  .card:global(.drop-after)::after {
    @apply -bottom-1.25;
  }
  /* Not around the dragged card itself, where dropping it changes nothing */
  .card.dragged:global(.drop-before)::before,
  .card.dragged:global(.drop-after)::after {
    @apply hidden;
  }
</style>
