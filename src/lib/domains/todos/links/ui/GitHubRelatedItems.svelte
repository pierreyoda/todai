<script lang="ts">
import type { Todo } from "../../model/types";
import { stateStatus } from "../model/github";
import { hasLink } from "../model/links";
import type { RelatedItem } from "../model/types";
import LinkChip from "./LinkChip.svelte";

/** The pull requests or issues a pull request or issue mentions, e.g. the issues it closes. */
type GitHubRelatedItemsProps = {
  items: readonly RelatedItem[];
  /** The todo whose link mentions them. */
  todo: Todo;
};
const { items, todo }: GitHubRelatedItemsProps = $props();
</script>

{#each items as item (item.url)}
  <span class="related-item">
    <LinkChip link={item} state={item.state} status={stateStatus(item.state)} />
    {#if hasLink(todo.links, item.url)}
      <span class="hint">Already on this todo</span>
    {/if}
  </span>
{/each}

<style lang="postcss">
  @reference "tailwindcss";

  .related-item {
    @apply flex items-center gap-2;
  }

  .hint {
    @apply text-xs/4 text-slate-400;
  }
</style>
