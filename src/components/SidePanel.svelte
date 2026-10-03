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

<section class="container">
  <div class="flex flex-col gap-4">
    <div class="flex items-center justify-between mb-2">
      <h2 class="section-title">Tags</h2>
      <Button
        size="xs"
        style="outline"
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
  <div class="flex flex-col">
    <hr />
    <h2 class="section-title">Settings</h2>
  </div>
  <TagUpsertModal bind:show={showTagCreationModal} />
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-60 h-full bg-gray-800 shadow-lg z-50;
    @apply flex flex-col justify-between text-center p-4;
  }

  .section-title {
    @apply text-sm font-semibold text-gray-400 uppercase tracking-wide;
  }

  hr {
    @apply border-b border-gray-300 dark:border-gray-700 py-4 mb-4;
  }
</style>
