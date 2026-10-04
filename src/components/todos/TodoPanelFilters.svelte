<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  import type { Tag } from "../../client/types";
  import TodoPanelAppliedFilter from "./TodoPanelAppliedFilter.svelte";

  type TodoPanelFiltersProps = {
    selectedTags: readonly Tag[];
    onDismissedTag: (tagId: Tag["id"]) => void;
    class?: ClassValue;
  };
  const {
    selectedTags,
    onDismissedTag,
    class: extraClass,
  }: TodoPanelFiltersProps = $props();
</script>

<div class={["container", extraClass]}>
  {#each selectedTags as tag (tag.id)}
    <TodoPanelAppliedFilter onDismiss={() => onDismissedTag(tag.id)} class="text-sm">
      <span style:color={tag.color}>{tag.name}</span>&nbsp;
      <span class="text-white">(tag)</span>
    </TodoPanelAppliedFilter>
  {/each}
</div>
