<script lang="ts">
  import { page } from "$app/state";

  import type { Tag } from "../client/types";
  import Button from "./common/Button.svelte";
  import Collapse from "./common/Collapse.svelte";
  import SidePanelTag from "./SidePanelTag.svelte";
  import IconSun from "./common/icons/IconSun.svelte";
  import SidePanelLink from "./layout/SidePanelLink.svelte";
  import TagUpsertModal from "./tags/TagUpsertModal.svelte";
  import TagsEmptyState from "./tags/TagsEmptyState.svelte";
  import IconCalendar from "./common/icons/IconCalendar.svelte";
  import IconFolderOpen from "./common/icons/IconFolderOpen.svelte";

  type SidePanelProps = {
    tags?: readonly Tag[] | "error" | "loading";
    selectedTagId?: Tag["id"] | null;
    onSelectedTagChanged?: (tagId: Tag["id"] | null) => void;
  };

  const { tags, selectedTagId, onSelectedTagChanged }: SidePanelProps =
    $props();

  let showTagCreationModal = $state(false);
  // The header's "+" button then gives way to a more inviting one, in the section's body
  const hasNoTags = $derived(Array.isArray(tags) && tags.length === 0);

  type CurrentPage = "home" | "calendar" | "trash";
  const currentPage = $derived<CurrentPage>(
    page.route.id === "/calendar" ? "calendar" : "home",
  );

  // TODO: add closing mechanism (click outside, button with icon)
</script>

<section class="container">
  <div class="flex flex-col gap-4">
    <SidePanelLink label="Today" routeId="/">
      {#snippet icon()}
        <IconSun class="text-white" />
      {/snippet}
    </SidePanelLink>
    <SidePanelLink label="Calendar" routeId="/calendar" reverse>
      {#snippet icon()}
        <IconCalendar class="text-white" />
      {/snippet}
    </SidePanelLink>
    {#if tags}
      <Collapse open>
        {#snippet summary()}
          <div class="flex items-center justify-between">
            <h2 class="section-title">Tags</h2>
            {#if !hasNoTags}
              <Button
                size="xs"
                style="outline"
                class="z-40"
                onclick={() => (showTagCreationModal = true)}>+</Button
              >
            {/if}
          </div>
        {/snippet}
        <div class="pt-4">
          {#if tags === "error"}
            ERROR
          {:else if tags === "loading"}
            LOADING
          {:else if hasNoTags}
            <TagsEmptyState onCreate={() => (showTagCreationModal = true)} />
          {:else}
            <ol class="flex flex-col gap-4 overflow-y-auto">
              {#each tags as tag (tag.id)}
                <SidePanelTag
                  {tag}
                  selected={tag.id === selectedTagId}
                  onSelectedChanged={(selected) =>
                    onSelectedTagChanged?.(selected ? tag.id : null)}
                />
              {/each}
            </ol>
          {/if}
        </div>
      </Collapse>
    {/if}
  </div>
  <div class="flex flex-col">
    <hr />
    <h2 class="section-title">Settings</h2>
    <SidePanelLink label="Workspaces" routeId="/settings/workspace">
      {#snippet icon()}
        <IconFolderOpen class="text-white" />
      {/snippet}
    </SidePanelLink>
  </div>
  <TagUpsertModal bind:show={showTagCreationModal} />
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    /* Never shrinks for the page's content, next to it */
    @apply w-60 h-full shrink-0 bg-gray-800 shadow-lg z-50;
    @apply flex flex-col justify-between text-center p-4;
  }

  :global(.page-link) {
    @apply w-full transition;
    &.current {
      @apply bg-gray-700 hover:bg-gray-500;
    }
  }

  .section-title {
    @apply text-sm font-semibold text-gray-400 uppercase tracking-wide;
  }

  hr {
    @apply border-b border-gray-300 dark:border-gray-700 mb-4;
  }
</style>
