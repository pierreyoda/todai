<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  import type { TodoLinkKind } from "../../client/types";
  import IconIssue from "../common/icons/IconIssue.svelte";
  import IconLink from "../common/icons/IconLink.svelte";
  import IconPullRequest from "../common/icons/IconPullRequest.svelte";

  type LinkKindIconProps = {
    kind: TodoLinkKind;
    class?: ClassValue;
  };
  const { kind, class: extraClass }: LinkKindIconProps = $props();
</script>

{#if kind === "github_pull"}
  <IconPullRequest class={extraClass} />
{:else if kind === "github_issue"}
  <IconIssue class={extraClass} />
{:else}
  <IconLink class={extraClass} />
{/if}
