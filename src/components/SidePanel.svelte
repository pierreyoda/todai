<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import type { Tag } from "../client/types";
  import { invokeClient } from "../client";
  import { tagKeys } from "../client/queries";
  import Button from "./common/Button.svelte";
  import SidePanelTag from "./SidePanelTag.svelte";
  import TagUpsertModal from "./tags/TagUpsertModal.svelte";

  type SidePanelProps = {
    tags: readonly Tag[] | "error" | "loading";
    selectedTagId: Tag["id"] | null;
    onSelectedTagChanged: (tagId: Tag["id"] | null) => void;
  };

  const { tags, selectedTagId, onSelectedTagChanged }: SidePanelProps =
    $props();

  let showTagCreationModal = $state(false);

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
  <TagUpsertModal bind:show={showTagCreationModal} />
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
