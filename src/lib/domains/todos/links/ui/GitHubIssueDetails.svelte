<script lang="ts">
import { formatRelativeTime } from "#shared/utils/timestamps.ts";
import type { Todo } from "../../model/types";
import type { IssueSnapshot } from "../model/types";
import GitHubRelatedItems from "./GitHubRelatedItems.svelte";
import LinkDetailsRow from "./LinkDetailsRow.svelte";

/** An issue's details, in its link's popover. */
type GitHubIssueDetailsProps = {
  snapshot: IssueSnapshot;
  /** The todo linked to it. */
  todo: Todo;
};
const { snapshot, todo }: GitHubIssueDetailsProps = $props();
</script>

{#if snapshot.closedBy.length > 0}
  <LinkDetailsRow label="Closed by" class="flex flex-col items-start gap-1">
    <GitHubRelatedItems items={snapshot.closedBy} {todo} />
  </LinkDetailsRow>
{/if}
<LinkDetailsRow
  label={snapshot.assignees.length > 1 ? "Assignees" : "Assignee"}
>
  {snapshot.assignees.join(", ") || "Nobody"}
</LinkDetailsRow>
{#if snapshot.milestone}
  <LinkDetailsRow label="Milestone">{snapshot.milestone}</LinkDetailsRow>
{/if}
<LinkDetailsRow label="Updated">
  {formatRelativeTime(snapshot.updatedAt)}
  {#if snapshot.author}· opened by {snapshot.author}{/if}
  · {snapshot.comments}
  {snapshot.comments === 1 ? "comment" : "comments"}
</LinkDetailsRow>
