<script lang="ts">
import { formatRelativeTime } from "#shared/utils/timestamps.ts";
import type { Todo } from "../../model/types";
import {
  checkName,
  describeChecks,
  describeReview,
  notableChecks,
} from "../model/github";
import type { PullRequestSnapshot } from "../model/types";
import GitHubRelatedItems from "./GitHubRelatedItems.svelte";
import LinkDetailsRow from "./LinkDetailsRow.svelte";

/** A pull request's details, in its link's popover. */
type GitHubPullDetailsProps = {
  snapshot: PullRequestSnapshot;
  /** The todo linked to it. */
  todo: Todo;
};
const { snapshot, todo }: GitHubPullDetailsProps = $props();

const CHECK_OUTCOMES = {
  passed: "Passed",
  failed: "Failed",
  running: "Running",
  skipped: "Skipped",
} as const;
</script>

{#if snapshot.checks}
  <LinkDetailsRow label="Checks" class="flex flex-col gap-0.5">
    <span>{describeChecks(snapshot.checks)}</span>
    <span class="check-list">
      {#each notableChecks(snapshot.checks) as check, i (i)}
        <span class={["check", check.outcome]}>
          {#if check.outcome === "passed"}
            <svg
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M3.5 8.5 6.5 11.5 12.5 4.5" />
            </svg>
          {:else if check.outcome === "failed"}
            <svg
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              aria-hidden="true"
            >
              <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
            </svg>
          {:else}
            <span class="ring" aria-hidden="true"></span>
          {/if}
          <!-- Told by its glyph, or else in words -->
          <span class="sr-only">{CHECK_OUTCOMES[check.outcome]}:</span>
          <span class="check-name">
            {checkName(
              check,
            )}{#if check.outcome === "running" && check.startedAt},
              started {formatRelativeTime(check.startedAt)}{/if}
          </span>
        </span>
      {/each}
    </span>
  </LinkDetailsRow>
{/if}
<LinkDetailsRow label="Review">{describeReview(snapshot)}</LinkDetailsRow>
{#if snapshot.closes.length > 0}
  <LinkDetailsRow label="Closes" class="flex flex-col items-start gap-1">
    <GitHubRelatedItems items={snapshot.closes} {todo} />
  </LinkDetailsRow>
{/if}
<LinkDetailsRow label="Changes" class="tabular-nums">
  <span class="text-green-300">
    +{snapshot.additions}<span class="sr-only"> lines added,</span>
  </span>
  <span class="text-red-300">
    −{snapshot.deletions}<span class="sr-only"> removed,</span>
  </span>
  · {snapshot.changedFiles}
  {snapshot.changedFiles === 1 ? "file" : "files"}
</LinkDetailsRow>
<LinkDetailsRow label="Updated">
  {formatRelativeTime(snapshot.updatedAt)}
  {#if snapshot.author}by {snapshot.author}{/if}
</LinkDetailsRow>

<style lang="postcss">
  @reference "tailwindcss";

  /* Scrolls past about seven checks, the next one cut to show there's more */
  .check-list {
    @apply flex max-h-40 flex-col gap-0.5 overflow-y-auto overscroll-contain;
  }

  .check {
    @apply flex items-center gap-1.5 text-slate-300;

    > :global(svg) {
      @apply size-3 shrink-0;
    }
    &.passed > :global(svg) {
      @apply text-green-300;
    }
    &.failed > :global(svg) {
      @apply text-red-300;
    }
    .ring {
      @apply mx-0.5 size-2 shrink-0 rounded-full border-[1.5px] border-amber-300;
    }
  }

  .check-name {
    @apply min-w-0 truncate;
  }
</style>
