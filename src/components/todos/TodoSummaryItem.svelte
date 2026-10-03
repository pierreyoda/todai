<script lang="ts">
  import type { Tag, Todo } from "../../client/types";
  import { createToggleTodoMutation } from "../../client/mutations";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";

  /** Slimmed down `TodoLineItem`: completeness toggle, title and tags, without edition. */
  type TodoSummaryItemProps = {
    item: Todo;
    tags: readonly Tag[];
  };
  const { item, tags }: TodoSummaryItemProps = $props();

  // Resolved from the tags list, so they follow tag renames without refetching todos
  const itemTags = $derived(tags.filter((tag) => item.tagIds.includes(tag.id)));

  // Follows the server state, but is checked/unchecked right away on click.
  let completed = $derived(item.completed);
  const toggleCompleted = createToggleTodoMutation(
    () => item,
    () => (completed = item.completed),
  );
</script>

<li class={["todo", completed && "completed"]}>
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
        <li class="tag">
          <span class="tag-color" style:background-color={tag.color}></span>
          {tag.name}
        </li>
      {/each}
    </ul>
  {/if}
</li>

<style lang="postcss">
  @reference "tailwindcss";

  .todo {
    @apply flex min-w-0 items-center gap-2 rounded-md px-2 py-1 hover:bg-white/5;
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
  }

  .tag-color {
    @apply size-2 rounded-full;
  }
</style>
