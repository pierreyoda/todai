<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import type { Snippet } from "svelte";

  import { invokeClient } from "../client";
  import type { Tag } from "../client/types";
  import { tagKeys } from "../client/queries";
  import TagUpsertModal from "./tags/TagUpsertModal.svelte";
  import IconTrash from "./common/icons/IconTrash.svelte";
  import IconPencilSquare from "./common/icons/IconPencilSquare.svelte";
  import {
    Dropdown,
    DropdownDivider,
    DropdownItem,
    DropdownMenu,
  } from "./common/dropdown";

  type SidePanelTagDropdownProps = {
    /** Bindable value. */
    open?: boolean;
    tag: Tag;
    onRename: () => void;
    /** Called once `tag` is deleted. */
    onDeleted: () => void;
    /** The tag's row, with a `DropdownTrigger` opening the menu. */
    children: Snippet;
  };
  let {
    open = $bindable(false),
    tag,
    onRename,
    onDeleted,
    children,
  }: SidePanelTagDropdownProps = $props();

  const queryClient = useQueryClient();
  const deleteTag = createMutation(() => ({
    mutationFn: () =>
      invokeClient({
        name: "delete_tag",
        args: { id: tag.id },
      }),
    onSuccess: () => {
      onDeleted();
      return queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));
  let confirmDeletion = $state(false);
  $effect(() => {
    if (!open) confirmDeletion = false;
  });

  let showColorModal = $state(false);
</script>

<Dropdown bind:open>
  {@render children()}
  <DropdownMenu placement="bottom-start">
    <DropdownItem keepOpen={false} onclick={onRename}>
      {#snippet icon()}
        <IconPencilSquare />
      {/snippet}
      Rename
    </DropdownItem>
    <DropdownItem keepOpen={false} onclick={() => (showColorModal = true)}>
      {#snippet icon()}
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="8" cy="8" r="5" fill={tag.color} />
        </svg>
      {/snippet}
      Change color
    </DropdownItem>
    <DropdownDivider />
    <DropdownItem
      onclick={() => {
        if (confirmDeletion) deleteTag.mutate();
        else confirmDeletion = true;
      }}
    >
      {#snippet icon()}
        <IconTrash class="text-red-400" />
      {/snippet}
      <span class="text-red-400">
        {#if confirmDeletion}
          Confirm
        {:else}
          Delete
        {/if}
      </span>
    </DropdownItem>
  </DropdownMenu>
</Dropdown>
<TagUpsertModal bind:show={showColorModal} existingTag={tag} mode="color" />
