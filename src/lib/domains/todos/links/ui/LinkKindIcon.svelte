<script lang="ts">
import type { ClassValue } from "svelte/elements";
import IconIssue from "#shared/ui/icons/IconIssue.svelte";
import IconIssueCompleted from "#shared/ui/icons/IconIssueCompleted.svelte";
import IconIssueNotPlanned from "#shared/ui/icons/IconIssueNotPlanned.svelte";
import IconLink from "#shared/ui/icons/IconLink.svelte";
import IconPullRequest from "#shared/ui/icons/IconPullRequest.svelte";
import IconPullRequestClosed from "#shared/ui/icons/IconPullRequestClosed.svelte";
import IconPullRequestDraft from "#shared/ui/icons/IconPullRequestDraft.svelte";
import IconPullRequestMerged from "#shared/ui/icons/IconPullRequestMerged.svelte";
import type { ItemState, TodoLinkKind } from "../model/types";

type LinkKindIconProps = {
  kind: TodoLinkKind;
  /** Where its pull request or issue stands, once known: picks the icon's variant, and colors it as GitHub does. */
  state?: ItemState;
  class?: ClassValue;
};
const { kind, state, class: extraClass }: LinkKindIconProps = $props();

const STATE_COLORS: Record<ItemState, string> = {
  open: "text-green-400",
  draft: "text-slate-400",
  merged: "text-purple-400",
  closed: "text-red-400",
  completed: "text-purple-400",
  not_planned: "text-slate-400",
  duplicate: "text-slate-400",
};
const iconClass = $derived([extraClass, state && STATE_COLORS[state]]);
</script>

{#if kind === "github_pull"}
  {#if state === "draft"}
    <IconPullRequestDraft class={iconClass} />
  {:else if state === "merged"}
    <IconPullRequestMerged class={iconClass} />
  {:else if state === "closed"}
    <IconPullRequestClosed class={iconClass} />
  {:else}
    <IconPullRequest class={iconClass} />
  {/if}
{:else if kind === "github_issue"}
  {#if state === "completed"}
    <IconIssueCompleted class={iconClass} />
  {:else if state === "not_planned" || state === "duplicate" || state === "closed"}
    <IconIssueNotPlanned class={iconClass} />
  {:else}
    <IconIssue class={iconClass} />
  {/if}
{:else}
  <IconLink class={extraClass} />
{/if}
