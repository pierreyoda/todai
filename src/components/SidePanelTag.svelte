<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import { invokeClient } from "../client";
  import type { Tag } from "../client/types";
  import { tagKeys } from "../client/queries";
  import { Debounced } from "../utils/debounced.svelte";
  import EditableText from "./common/EditableText.svelte";

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
  let editedName = $state(tag.name);
  const debouncedEditedName = new Debounced(() => editedName, 500);
  $effect(() => {
    const name = debouncedEditedName.current;
    // Also runs on mount, with the unchanged name
    if (name === tag.name) return;
    // `mutate` reads the mutation's state, which it then updates: untracked so the effect doesn't loop
    untrack(() => updateTagName.mutate(name));
  });
</script>

<!-- TODO: fix a11y warning -->
<li
  class={["tag-item", selected && "selected"]}
  on:click={() => onSelectedChanged(!selected)}
>
  <span class="count-badge">
    {tag.linkedTodosCount ?? 0}<span class="sr-only"> todos</span>
  </span>
  <EditableText
    as="h4"
    bind:value={editedName}
    label={`Name for tag with current name "${tag.name}"`}
    class="flex-1 text-white text-sm"
  />
  <div class="color-marker" style:background-color={tag.color}></div>
</li>

<style lang="postcss">
  @reference "tailwindcss";

  .tag-item {
    @apply w-full flex items-center justify-between gap-2;
    .count-badge {
      /* Layout: fixed minimum width so single and double digits line up */
      @apply inline-flex min-w-6 shrink-0 items-center justify-center rounded-md px-1.5 py-0.5;
      /* Typography */
      @apply text-xs/4 font-medium text-gray-400 tabular-nums;
      /* Surface, matching the translucent fields and menus */
      @apply bg-white/5 ring-1 ring-white/10 ring-inset;
      /* Forced colors mode */
      @apply forced-colors:outline;
    }
    .color-marker {
      @apply w-4 h-4 shrink-0 rounded-full;
    }
  }
</style>
