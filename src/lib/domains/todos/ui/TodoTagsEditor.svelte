<script lang="ts">
import type { UUID } from "node:crypto";
import { createQuery } from "@tanstack/svelte-query";
import { tagsQueryOptions } from "#domains/tags";
import {
  Dropdown,
  DropdownItem,
  DropdownMenu,
  DropdownTrigger,
} from "#shared/ui/dropdown/index.ts";
import IconPlus from "#shared/ui/icons/IconPlus.svelte";
import IconXMark from "#shared/ui/icons/IconXMark.svelte";
  import Button from "#shared/ui/Button.svelte";

/**
 * The tags of a todo, as a field of its form: added and removed here, then saved with the rest once it's submitted. The
 * same for a todo to create, which has none to save them to yet.
 */
type TodoTagsEditorProps = {
  /** Bindable. */
  tagIds: readonly UUID[];
};
let { tagIds = $bindable() }: TodoTagsEditorProps = $props();

const titleId = $props.id();
const tags = createQuery(() => tagsQueryOptions);
/** In the order of the tags, as in the lists. */
const assignedTags = $derived(
  tags.data?.filter(({ id }) => tagIds.includes(id)) ?? [],
);

const assign = (id: UUID, assigned: boolean) => {
  tagIds = assigned ? [...tagIds, id] : tagIds.filter((tagId) => tagId !== id);
};
</script>

<section class="tags" aria-labelledby={titleId}>
  <div class="head">
    <h3 id={titleId} class="title">Tags</h3>
    <Dropdown>
      <DropdownTrigger>
        {#snippet children(trigger)}
          <Button
            style="plain"
            {...trigger}
            class="choose"
            title="Choose its tags"
            aria-label="Choose its tags"
          >
            <IconPlus />
        </Button>
        {/snippet}
      </DropdownTrigger>
      <!-- Left open, to choose several -->
      <DropdownMenu placement="bottom-end">
        {#each tags.data ?? [] as tag (tag.id)}
          {@const assigned = tagIds.includes(tag.id)}
          <DropdownItem checked={assigned} onclick={() => assign(tag.id, !assigned)}>
            <span class="dot mr-2" style:--tag-color={tag.color}></span>
            {tag.name}
          </DropdownItem>
        {:else}
          <DropdownItem disabled>No tags yet</DropdownItem>
        {/each}
      </DropdownMenu>
    </Dropdown>
  </div>
  {#if assignedTags.length > 0}
    <ul class="list">
      {#each assignedTags as { id, name, color } (id)}
        <li class="tag">
          <span class="dot" style:--tag-color={color}></span>
          <span class="name">{name}</span>
          <button
            type="button"
            class="remove"
            title="Remove"
            aria-label={`Remove the tag ${name}`}
            onclick={() => assign(id, false)}
          >
            <IconXMark class="size-3.5" />
          </button>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="empty">No tags</p>
  {/if}
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .tags {
    @apply flex flex-col gap-1.5;
  }

  /* The title on the left, the button choosing the tags on the right */
  .head {
    @apply flex items-center justify-between gap-3;
  }

  .title {
    /* Like the fields' labels */
    @apply text-sm font-medium text-slate-300;
  }

  .list {
    @apply flex flex-wrap gap-1.5;
  }

  .tag {
    @apply flex min-w-0 items-center gap-1.5 rounded-full bg-white/5 py-0.5 pr-0.5 pl-2.5 ring-1 ring-white/10;
  }

  /* Inline in the menu, before the tag's name */
  .dot {
    @apply inline-block size-2.5 shrink-0 rounded-full bg-(--tag-color) align-middle;
  }

  .name {
    @apply min-w-0 truncate text-sm/5 text-white;
  }

  .empty {
    @apply text-sm/5 text-slate-500;
  }

  .choose,
  .remove {
    @apply grid shrink-0 cursor-pointer place-items-center text-slate-400;
    @apply transition-colors hover:bg-white/10 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }

  .choose {
    @apply size-7 rounded-md;
  }

  .remove {
    @apply size-5 rounded-full;
  }
</style>
