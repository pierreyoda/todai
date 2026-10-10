<script lang="ts">
  import type { Tag } from "../client/types";
  import { dateToTodaiDate, formatDay } from "../utils/dates";
  import Collapse from "./common/Collapse.svelte";
  import SidePanelTag from "./SidePanelTag.svelte";
  import IconSun from "./common/icons/IconSun.svelte";
  import IconPlus from "./common/icons/IconPlus.svelte";
  import SidePanelLink from "./layout/SidePanelLink.svelte";
  import TagUpsertModal from "./tags/TagUpsertModal.svelte";
  import TagsEmptyState from "./tags/TagsEmptyState.svelte";
  import IconCalendar from "./common/icons/IconCalendar.svelte";
  import IconFolderOpen from "./common/icons/IconFolderOpen.svelte";
  import IconArchiveBox from "./common/icons/IconArchiveBox.svelte";
  import SidePanelWorkspace from "./layout/SidePanelWorkspace.svelte";
  import IconExclamationTriangle from "./common/icons/IconExclamationTriangle.svelte";

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

  const today = formatDay(dateToTodaiDate(new Date()));

  // TODO: add closing mechanism (click outside, button with icon)
</script>

<aside class="side-panel">
  <SidePanelWorkspace />
  <nav aria-label="Pages">
    <ul class="links">
      <li>
        <SidePanelLink label="Today" routeId="/" hint={today}>
          {#snippet icon()}
            <IconSun />
          {/snippet}
        </SidePanelLink>
      </li>
      <li>
        <SidePanelLink label="Calendar" routeId="/calendar">
          {#snippet icon()}
            <IconCalendar />
          {/snippet}
        </SidePanelLink>
      </li>
    </ul>
  </nav>
  <!-- The only part to scroll, so that the pages and settings stay in reach -->
  <div class="scrollable">
    {#if tags}
      <section class="tags-section">
        <Collapse open headingLevel={2}>
          {#snippet summary()}
            <span class="section-title">Tags</span>
          {/snippet}
          <div class="pt-1">
            {#if tags === "error"}
              <p class="status error" role="alert">
                <IconExclamationTriangle class="shrink-0" />
                Could not load the tags.
              </p>
            {:else if tags === "loading"}
              <p class="sr-only" role="status">Loading the tags…</p>
              <!-- Placeholder rows, about as wide as tags names -->
              <ul class="skeleton" aria-hidden="true">
                {#each [70, 45, 60] as width (width)}
                  <li>
                    <span class="dot"></span>
                    <span class="bar" style:width="{width}%"></span>
                  </li>
                {/each}
              </ul>
            {:else if hasNoTags}
              <TagsEmptyState onCreate={() => (showTagCreationModal = true)} />
            {:else}
              <ol class="tags">
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
        <!-- Next to the toggle rather than within it: buttons can't be nested -->
        {#if !hasNoTags}
          <button
            type="button"
            class="add-tag"
            title="Create a tag"
            aria-label="Create a tag"
            onclick={() => (showTagCreationModal = true)}
          >
            <IconPlus />
          </button>
        {/if}
        <TagUpsertModal bind:show={showTagCreationModal} />
      </section>
    {/if}
  </div>
  <footer class="settings">
    <h2 class="section-title">Settings</h2>
    <SidePanelLink label="Workspaces" routeId="/settings/workspace">
      {#snippet icon()}
        <IconFolderOpen />
      {/snippet}
    </SidePanelLink>
    <SidePanelLink label="Backups" routeId="/settings/backups">
      {#snippet icon()}
        <IconArchiveBox />
      {/snippet}
    </SidePanelLink>
  </footer>
</aside>

<style lang="postcss">
  @reference "tailwindcss";

  .side-panel {
    /* Layout: never shrinks for the page's content, next to it */
    @apply flex h-full w-64 shrink-0 flex-col gap-4 p-3;
    /* Surface: a shade darker than the page, which reads as the app's frame. Exposed for the elements cut out of it. */
    @apply [--side-panel-bg:color-mix(in_oklab,var(--color-slate-900)_55%,var(--color-slate-950))];
    @apply border-r border-white/5 bg-(--side-panel-bg);
  }

  .links {
    @apply flex flex-col gap-0.5;
  }

  .scrollable {
    /* Bleeds into the panel's padding, so that the scrollbar sits against its edge */
    @apply -mx-3 min-h-0 flex-1 overflow-y-auto px-3;
  }

  .tags-section {
    @apply relative;
    /* Lines up the chevron and title with the links' icons and labels */
    :global(.toggle) {
      @apply gap-3 px-2.5;
    }
  }

  .section-title {
    @apply text-xs/6 font-semibold tracking-wider text-slate-400 uppercase;
  }

  .add-tag {
    /* Layout: centered on the section's toggle */
    @apply absolute top-1.5 right-1.5 grid size-6 cursor-pointer place-items-center rounded-md;
    /* States */
    @apply text-slate-400 transition-colors hover:bg-white/10 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }

  .tags {
    @apply flex flex-col gap-0.5;
  }

  .status {
    @apply flex items-center gap-2 px-2.5 py-1.5 text-sm text-slate-400;
    &.error {
      @apply text-red-400;
    }
  }

  .skeleton {
    @apply flex flex-col gap-0.5;
    li {
      @apply flex items-center gap-3 px-2.5 py-1.5;
    }
    .dot {
      /* Centered where the tags' dots are */
      @apply mx-0.75 size-2.5 shrink-0 rounded-full bg-white/10;
    }
    .bar {
      @apply h-2.5 rounded-full bg-white/10;
    }
    .dot,
    .bar {
      @apply my-1.75 animate-pulse motion-reduce:animate-none;
    }
  }

  .settings {
    @apply flex flex-col gap-1 border-t border-white/5 pt-3;
    .section-title {
      @apply px-2.5;
    }
  }
</style>
