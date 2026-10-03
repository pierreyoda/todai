<script lang="ts">
  import type { Tag } from "../client/types";

  type SidePanelTagProps = {
    tag: Tag;
    selected: boolean;
    onSelectedChanged: (selected: boolean) => void;
  };

  const { tag, selected, onSelectedChanged }: SidePanelTagProps = $props();
</script>

<!-- TODO: fix a11y warning -->
<li
  class={["tag-item", selected && "selected"]}
  on:click={() => onSelectedChanged(!selected)}
>
  <h4 class="name">{tag.name}</h4>
  <div class="color-marker" style:background-color={tag.color}></div>
</li>

<style lang="postcss">
  @reference "tailwindcss";

  .tag-item {
    @apply w-full flex items-center justify-between;
    .name {
      @apply text-white text-sm;
    }
    .color-marker {
      @apply w-4 h-4 rounded-full;
    }
  }
</style>
