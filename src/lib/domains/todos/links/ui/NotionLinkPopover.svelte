<script lang="ts">
import { formatRelativeTime } from "#shared/utils/timestamps.ts";
import type { Todo } from "../../model/types";
import { formatDue } from "../model/notion";
import type { TodoLink } from "../model/types";
import LinkDetailsRow from "./LinkDetailsRow.svelte";
import LinkPopover from "./LinkPopover.svelte";
import NotionColorDot from "./NotionColorDot.svelte";

/** A Notion page's (or database's) details, as linked to a todo. */
type NotionLinkPopoverProps = {
  /** Bindable. */
  open: boolean;
  /** A Notion link of `todo`. */
  link: TodoLink;
  todo: Todo;
  /** What it's shown next to: its chip. */
  anchor: HTMLElement | undefined;
};
let {
  open = $bindable(),
  link,
  todo,
  anchor,
}: NotionLinkPopoverProps = $props();

const snapshot = $derived(
  link.snapshot?.type === "notion_page" ||
    link.snapshot?.type === "notion_database"
    ? link.snapshot
    : undefined,
);
const page = $derived(snapshot?.type === "notion_page" ? snapshot : undefined);
const isDatabase = $derived(snapshot?.type === "notion_database");
</script>

{#snippet where()}
  {#if isDatabase}
    Notion database
  {:else}
    Notion page
    {#if page?.database?.title}
      · {page.database.title}
    {/if}
    {#if page?.uniqueId}
      · <span class="tabular-nums">{link.label}</span>
    {/if}
  {/if}
{/snippet}

{#snippet extras()}
  {#if page}
    {#each page.tags as tag (tag.name)}
      <span class="tag">
        <NotionColorDot color={tag.color} />
        {tag.name}
      </span>
    {/each}
  {/if}
{/snippet}

{#snippet fetchErrorNotice()}
  Notion doesn't let this workspace's token see it: it may be someone else's
  private page, in another Notion workspace than the token's, or deleted for
  good.
{/snippet}

{#snippet details()}
  {#if snapshot?.inTrash}
    <LinkDetailsRow label="Trash">
      It's in Notion's trash: restore it there, or remove the link.
    </LinkDetailsRow>
  {/if}
  {#if page}
    {#if page.assignees.length > 0}
      <LinkDetailsRow
        label={page.assignees.length > 1 ? "Assignees" : "Assignee"}
      >
        {page.assignees.join(", ")}
      </LinkDetailsRow>
    {/if}
    {#if page.due}
      <LinkDetailsRow label="Due">{formatDue(page.due)}</LinkDetailsRow>
    {/if}
  {/if}
  {#if snapshot}
    <LinkDetailsRow label="Updated">
      {formatRelativeTime(snapshot.lastEditedAt)}
    </LinkDetailsRow>
  {/if}
{/snippet}

<LinkPopover
  bind:open
  {link}
  {todo}
  provider="notion"
  {anchor}
  iconVariant={isDatabase ? "database" : undefined}
  {where}
  {extras}
  {fetchErrorNotice}
  details={snapshot ? details : undefined}
/>

<style lang="postcss">
  @reference "tailwindcss";

  .tag {
    @apply inline-flex items-center gap-1 text-xs/4 text-slate-300;
  }
</style>
