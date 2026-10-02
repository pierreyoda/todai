<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import type { Tag } from "../client/types";
  import { invokeClient } from "../client";
  import { tagKeys } from "../client/queries";
  import Modal from "./common/Modal.svelte";
  import Button from "./common/Button.svelte";
  import SidePanelTag from "./SidePanelTag.svelte";
  import TagForm from "./tags/TagForm.svelte";

  type SidePanelProps = {
    tags: readonly Tag[] | "error" | "loading";
    selectedTagId: Tag["id"] | null;
    onSelectedTagChanged: (tagId: Tag["id"] | null) => void;
  };

  const { tags, selectedTagId, onSelectedTagChanged }: SidePanelProps =
    $props();

  let showTagCreationModal = $state(false);

  const queryClient = useQueryClient();
  const createTag = createMutation(() => ({
    mutationFn: (newTag: Pick<Tag, "name" | "color">) =>
      invokeClient({
        name: "create_tag",
        args: newTag,
      }),
    onSuccess: () => {
      showTagCreationModal = false;
      queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));

  // TODO: add closing mechanism (click outside, button with icon)
</script>

<section class="container pt-4 pb-2">
  <div class="flex flex-col gap-4 p-2">
    <div class="flex items-center justify-between">
      <h2 class="section-title">Tags</h2>
      <Button
        onclick={() => {
          if (!showTagCreationModal) {
            showTagCreationModal = true;
          }
        }}>+</Button
      >
    </div>
    {#if tags === "error"}
      ERROR
    {:else if tags === "loading"}
      LOADING
    {:else}
      <ol class="flex flex-col gap-4 overflow-y-auto">
        {#each tags as tag (tag.id)}
          <SidePanelTag
            {tag}
            selected={tag.id === selectedTagId}
            onSelectedChanged={(selected) =>
              onSelectedTagChanged(selected ? tag.id : null)}
          />
        {/each}
      </ol>
    {/if}
  </div>
  <div class="p-4 flex flex-col">
    <div class="divider"></div>
    <h2 class="section-title">Settings</h2>
  </div>
  <div class="modal-container">
    <Modal bind:show={showTagCreationModal}>
      {#snippet header()}
        Create a new tag
      {/snippet}
      <TagForm
        data={{ name: "", color: "" }}
        onSubmit={(name, color) => createTag.mutate({ name, color })}
      />
    </Modal>
  </div>
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-60 h-full bg-gray-800 shadow-lg z-50;
    @apply flex flex-col justify-between text-center;
  }

  .section-title {
    @apply text-sm font-semibold text-gray-400 uppercase tracking-wide px-4;
  }

  .divider {
    @apply border-b border-gray-300 dark:border-gray-700 py-4 mb-4;
  }
</style>
