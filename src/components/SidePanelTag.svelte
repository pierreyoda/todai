<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";

  import { invokeClient } from "../client";
  import type { Tag } from "../client/types";
  import { tagKeys } from "../client/queries";
  import { contextMenu } from "../utils/contextMenu";
  import EditableText from "./common/EditableText.svelte";
  import { DropdownTrigger } from "./common/dropdown";
  import IconEllipsisVertical from "./common/icons/IconEllipsisVertical.svelte";
  import SidePanelTagDropdown from "./SidePanelTagDropdown.svelte";

  type SidePanelTagProps = {
    tag: Tag;
    selected: boolean;
    onSelectedChanged: (selected: boolean) => void;
  };
  const { tag, selected, onSelectedChanged }: SidePanelTagProps = $props();

  const queryClient = useQueryClient();
  const updateTagName = createMutation(() => ({
    mutationFn: (newName: string) =>
      invokeClient({
        name: "update_tag",
        args: { id: tag.id, name: newName },
      }),
    onSuccess: () => {
      // Todos reference tags by ID, so the tags list is the only one to refresh
      return queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));
  const saveName = (name: string) => {
    // Also called when confirming the unchanged name
    if (name !== tag.name) updateTagName.mutate(name);
  };

  let renaming = $state(false);
  let selectButton = $state<HTMLButtonElement>();
  const startRenaming = () => (renaming = true);
  const stopRenaming = async () => {
    renaming = false;
    await tick();
    // Its field replaced the tag's button: back on it, unless the focus moved elsewhere (e.g. clicking away)
    if (document.activeElement === document.body) selectButton?.focus();
  };

  let menuOpen = $state(false);
</script>

<li
  class={["tag", selected && "selected", renaming && "renaming"]}
  style:--tag-color={tag.color}
>
  <SidePanelTagDropdown
    bind:open={menuOpen}
    {tag}
    onRename={startRenaming}
    onDeleted={() => {
      // Its todos would otherwise stay filtered by a tag that no longer exists
      if (selected) onSelectedChanged(false);
    }}
  >
    <!-- Not while renaming: right-clicking its field shows the field's own menu (cut, copy, paste...) -->
    <div
      class="row"
      {@attach !renaming && contextMenu(() => (menuOpen = true))}
    >
      {#if renaming}
        <div class="item">
          <span class="dot" aria-hidden="true"></span>
          <EditableText
            as="span"
            bind:value={() => tag.name, saveName}
            bind:editing={
              () => renaming, (editing) => !editing && stopRenaming()
            }
            label={`Name of tag "${tag.name}"`}
            class="min-w-0 flex-1 text-sm/6 font-medium text-white"
          />
        </div>
      {:else}
        <button
          type="button"
          bind:this={selectButton}
          class="item select"
          aria-pressed={selected}
          onclick={(event) => {
            // The second click of a double click, which renames it, would otherwise undo the first
            if (event.detail > 1) return;
            onSelectedChanged(!selected);
          }}
          ondblclick={startRenaming}
        >
          <span class="dot" aria-hidden="true"></span>
          <span class="name" title={tag.name}>{tag.name}</span>
          {#if tag.linkedTodosCount !== undefined}
            <span
              class={["count", tag.linkedTodosCount === 0 && "empty"]}
              title={`${tag.linkedTodosCount} todos`}
            >
              {tag.linkedTodosCount}<span class="sr-only"> todos</span>
            </span>
          {/if}
        </button>
        <DropdownTrigger>
          {#snippet children(trigger)}
            <button
              {...trigger}
              class="menu-button"
              title="Options"
              aria-label={`Options for tag "${tag.name}"`}
            >
              <IconEllipsisVertical />
            </button>
          {/snippet}
        </DropdownTrigger>
      {/if}
    </div>
  </SidePanelTagDropdown>
</li>

<style lang="postcss">
  @reference "tailwindcss";

  .row {
    @apply relative;
  }

  /* The tag's button, or its name's field while renaming it */
  .item {
    /* Layout: dots and names line up with the links' icons and labels */
    @apply flex w-full items-center gap-3 rounded-md px-2.5 py-1.5 text-left;
  }

  .select {
    @apply cursor-pointer transition-colors hover:bg-white/5;
    /* Inset: the section clips what overflows it */
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500;
  }

  .dot {
    /* Centered in the same width as the links' icons */
    @apply mx-0.75 size-2.5 shrink-0 rounded-full bg-(--tag-color) transition-shadow;
    /* Light inner edge, so that the darkest colors stand out from the panel */
    @apply shadow-[inset_0_0_0_1px_--theme(--color-white/15%)];
  }

  .name {
    @apply min-w-0 flex-1 truncate text-sm/6 text-slate-300 transition-colors;
    .select:hover & {
      @apply text-white;
    }
  }

  .count {
    /* Layout: fixed minimum width so single and double digits line up */
    @apply min-w-5 shrink-0 text-right transition-opacity;
    /* Typography */
    @apply text-xs font-medium text-slate-400 tabular-nums;
    &.empty {
      @apply text-slate-600;
    }
  }

  .menu-button {
    /* Layout: over the count, which gives way to it */
    @apply absolute top-1/2 right-1.5 grid size-6 -translate-y-1/2 cursor-pointer place-items-center rounded-md;
    /* States: only revealed with the tag hovered or focused, or its menu open */
    @apply text-slate-400 opacity-0 transition hover:bg-white/10 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500;
    @apply aria-expanded:bg-white/10 aria-expanded:text-white;
  }

  .row:hover,
  .row:has(:focus-visible),
  .row:has(.menu-button[aria-expanded="true"]) {
    .menu-button {
      @apply opacity-100;
    }
    .count {
      @apply opacity-0;
    }
  }

  /* Tinted with the tag's color, with a thin accent on the leading edge, like the todos it highlights */
  .selected {
    .select {
      @apply bg-[color-mix(in_oklab,var(--tag-color)_15%,transparent)] shadow-[inset_2px_0_0_var(--tag-color)];
      @apply hover:bg-[color-mix(in_oklab,var(--tag-color)_22%,transparent)];
    }
    .dot {
      @apply ring-3 ring-(--tag-color)/25;
    }
    .name {
      @apply font-medium text-white;
    }
    .count {
      @apply text-white;
    }
  }

  /* Its name being edited, in place */
  .renaming .item {
    @apply bg-white/5 ring-1 ring-blue-500/60 ring-inset;
  }
</style>
