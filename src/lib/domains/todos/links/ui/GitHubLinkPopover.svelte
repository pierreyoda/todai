<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { activeWorkspaceQueryOptions } from "#domains/workspaces";
import type { Todo } from "../../model/types";
import { itemState } from "../model/github";
import type { TodoLink } from "../model/types";
import GitHubIssueDetails from "./GitHubIssueDetails.svelte";
import GitHubPullDetails from "./GitHubPullDetails.svelte";
import LinkPopover from "./LinkPopover.svelte";

/** A GitHub pull request's or issue's details, as linked to a todo. */
type GitHubLinkPopoverProps = {
  /** Bindable. */
  open: boolean;
  /** A GitHub link of `todo`. */
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
}: GitHubLinkPopoverProps = $props();

const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);

const snapshot = $derived(link.snapshot);
const github = $derived(link.github);
const isIssue = $derived(
  snapshot ? snapshot.type === "issue" : link.kind === "github_issue",
);
</script>

{#snippet where()}
  {isIssue ? "Issue" : "Pull request"}
  {#if github}
    · {github.owner}/{github.repo}
    <span class="number">#{github.number}</span>
  {/if}
{/snippet}

{#snippet extras()}
  {#if snapshot?.type === "pull_request"}
    <span class="branches">{snapshot.headRef} → {snapshot.baseRef}</span>
  {:else if snapshot}
    {#each snapshot.labels as label (label.name)}
      <span class="label">
        <span
          class="dot"
          style:background-color={label.color}
          aria-hidden="true"
        ></span>
        {label.name}
      </span>
    {/each}
  {/if}
{/snippet}

{#snippet fetchErrorNotice(fetchError: NonNullable<TodoLink["fetchError"]>)}
  {#if fetchError === "sso_required"}
    Its organization requires single sign-on: authorize the GitHub CLI's
    token for it on github.com, then refresh.
  {:else}
    GitHub doesn't let {activeWorkspace.data?.githubLogin ?? "this account"}
    see it: it may be private, deleted, or not approved by its organization.
  {/if}
{/snippet}

{#snippet details()}
  {#if snapshot?.type === "pull_request"}
    <GitHubPullDetails {snapshot} {todo} />
  {:else if snapshot?.type === "issue"}
    <GitHubIssueDetails {snapshot} {todo} />
  {/if}
{/snippet}

<LinkPopover
  bind:open
  {link}
  {todo}
  provider="github"
  {anchor}
  iconState={snapshot && itemState(snapshot)}
  {where}
  {extras}
  {fetchErrorNotice}
  details={snapshot ? details : undefined}
/>

<style lang="postcss">
  @reference "tailwindcss";

  .number {
    @apply tabular-nums;
  }

  .branches {
    @apply min-w-0 truncate text-xs/4 text-slate-400;
  }

  .label {
    @apply inline-flex items-center gap-1 text-xs/4 text-slate-300;
  }

  .dot {
    @apply size-2 shrink-0 rounded-full;
  }
</style>
