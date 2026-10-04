<script lang="ts">
  import type { Tag, Todo } from "../../client/types";
  import { createToggleTodoMutation } from "../../client/mutations";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";
  import TodoDropdownMenu from "./TodoDropdownMenu.svelte";
  import { contextMenu } from "../../utils/contextMenu";
  import IconPencilSquare from "../common/icons/IconPencilSquare.svelte";
  import Button from "../common/Button.svelte";
  import TodoUpsertModal from "./TodoUpsertModal.svelte";

  /** Slimmed down `TodoLineItem`: completeness toggle, title and tags, edition on right click. */
  type TodoSummaryItemProps = {
    item: Todo;
    tags: readonly Tag[];
    /** Todos with this tag are highlighted in its color. */
    selectedTagId?: Tag["id"] | null;
  };
  const { item, tags, selectedTagId = null }: TodoSummaryItemProps = $props();

  // Resolved from the tags list, so they follow tag renames without refetching todos
  const itemTags = $derived(tags.filter((tag) => item.tagIds.includes(tag.id)));
  const highlightTag = $derived(
    itemTags.find((tag) => tag.id === selectedTagId),
  );

  // Follows the server state, but is checked/unchecked right away on click.
  let completed = $derived(item.completed);
  const toggleCompleted = createToggleTodoMutation(
    () => item,
    () => (completed = item.completed),
  );

  let lineRef = $state<HTMLLIElement>();
  let openedDropdownMenu = $state(false);

  let showEditDialog = $state(false);
</script>

<li
  bind:this={lineRef}
  class={["todo", completed && "completed", highlightTag && "highlighted"]}
  style:--tag-color={highlightTag?.color}
  {@attach contextMenu(() => (openedDropdownMenu = true))}
>
  <FieldCheckbox
    label={`Completed: ${item.title}`}
    hideLabel
    bind:checked={completed}
    onchange={(event) => toggleCompleted.mutate(event.currentTarget.checked)}
  />
  <span class="title" title={item.title}>{item.title}</span>
  {#if itemTags.length > 0}
    <ul class="tags" aria-label="Tags">
      {#each itemTags as tag (tag.id)}
        <li class={["tag", tag.id === selectedTagId && "selected"]}>
          <span class="tag-color" style:background-color={tag.color}></span>
          {tag.name}
        </li>
      {/each}
    </ul>
  {/if}
  <div class="edit-action-container" title="Edit">
    <Button style="plain" onclick={() => (showEditDialog = true)}>
      <IconPencilSquare class="text-white hover:text-gray-300" />
    </Button>
  </div>
</li>
<TodoDropdownMenu
  bind:open={openedDropdownMenu}
  anchor={lineRef}
  {item}
  {tags}
  inCalendar
/>
<TodoUpsertModal bind:show={showEditDialog} existing={item} />

<style lang="postcss">
  @reference "tailwindcss";

  .todo {
    @apply flex min-w-0 items-center gap-2 rounded-md px-2 py-1 transition hover:bg-white/5;
    .edit-action-container {
      @apply flex items-center gap-1 opacity-0;
    }
    &:hover {
      .edit-action-container {
        @apply opacity-100;
      }
    }
  }

  /* Tinted with the selected tag's color, with a thin accent on the leading edge */
  .highlighted {
    @apply bg-[color-mix(in_oklab,var(--tag-color)_12%,transparent)] shadow-[inset_2px_0_0_var(--tag-color)];
    @apply hover:bg-[color-mix(in_oklab,var(--tag-color)_20%,transparent)];
  }

  .title {
    @apply min-w-0 flex-1 truncate text-sm text-white;
  }

  .completed .title {
    @apply text-slate-500 line-through;
  }

  .tags {
    @apply flex shrink-0 items-center gap-2;
  }

  .tag {
    @apply flex items-center gap-1 text-xs text-slate-400;
    &.selected {
      @apply text-white;
    }
  }

  .tag-color {
    @apply size-2 rounded-full;
  }
</style>
