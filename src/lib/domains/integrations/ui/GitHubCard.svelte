<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { describeProblem } from "#domains/todos";
import type { Workspace } from "#domains/workspaces";
import Button from "#shared/ui/Button.svelte";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import IconPullRequest from "#shared/ui/icons/IconPullRequest.svelte";
import { formatTime } from "#shared/utils/timestamps.ts";
import {
  createDisconnectGitHubMutation,
  createRefindGhMutation,
} from "../api/mutations";
import {
  ghStatusQueryOptions,
  githubApiStatusQueryOptions,
} from "../api/queries";
import { githubConnectionState } from "../model/connection";
import GitHubConnectModal from "./GitHubConnectModal.svelte";
import IntegrationCard from "./IntegrationCard.svelte";

type GitHubCardProps = {
  workspace: Workspace;
};
const { workspace }: GitHubCardProps = $props();

const ghStatus = createQuery(() => ghStatusQueryOptions);
const refind = createRefindGhMutation();
const disconnect = createDisconnectGitHubMutation();
const connection = $derived(
  githubConnectionState(workspace.githubLogin, ghStatus.data),
);
const needsAttention = $derived(
  connection === "signed_out" || connection === "gh_missing",
);
const cardState = $derived(
  connection === "connected"
    ? "connected"
    : needsAttention
      ? "attention"
      : connection === "checking"
        ? "checking"
        : "not_connected",
);
// GitHub's side, once the GitHub CLI's is fine: which also tells whether GitHub accepts its token
const apiStatus = createQuery(() => ({
  ...githubApiStatusQueryOptions(workspace.id),
  enabled: connection === "connected",
}));
const rateLimit = $derived(apiStatus.data?.rateLimit);
const apiProblem = $derived(
  apiStatus.data?.problem && describeProblem("github", apiStatus.data.problem),
);

let showConnectModal = $state(false);
</script>

<IntegrationCard
  name="GitHub"
  subtitle="Pull requests and issues: state, checks and reviews"
  state={cardState}
  onConnect={() => (showConnectModal = true)}
>
  {#snippet logo()}
    <IconPullRequest />
  {/snippet}

  {#if workspace.githubLogin}
    <dl class="details">
      <dt>Account</dt>
      <dd>{workspace.githubLogin}, from the GitHub CLI</dd>
      {#if ghStatus.data?.path}
        <dt>GitHub CLI</dt>
        <dd class="path">{ghStatus.data.path}</dd>
      {/if}
      {#if connection === "connected"}
        <dt>Rate limit</dt>
        <dd>
          {#if rateLimit}
            {rateLimit.remaining.toLocaleString("en-US")} of
            {rateLimit.limit.toLocaleString("en-US")} points left, until
            {formatTime(rateLimit.resetAt)}
          {:else if apiProblem}
            <span class="text-amber-300">{apiProblem.label}</span>
          {:else if apiStatus.isError}
            <span class="text-slate-400">Unknown</span>
          {:else}
            <span class="text-slate-400">Asking GitHub…</span>
          {/if}
        </dd>
      {/if}
    </dl>
    {#if apiProblem && apiStatus.data?.problem?.kind !== "offline"}
      <div class="problem" role="alert">
        <p class="problem-title">GitHub refused to answer</p>
        <p class="problem-text">{apiProblem.detail}</p>
      </div>
    {/if}
    {#if connection === "signed_out"}
      <div class="problem" role="alert">
        <p class="problem-title">
          The GitHub CLI isn't signed in to {workspace.githubLogin} anymore
        </p>
        <p class="problem-text">
          Sign in again in a terminal with <code>gh auth login</code>, then
          check again. Until then, this workspace's links only show what their
          URL tells.
        </p>
      </div>
    {:else if connection === "gh_missing"}
      <div class="problem" role="alert">
        <p class="problem-title">The GitHub CLI wasn't found</p>
        <p class="problem-text">
          Install it (<code>brew install gh</code>), sign in with
          <code>gh auth login</code>, then check again.
        </p>
      </div>
    {/if}
    <div class="actions">
      {#if needsAttention}
        <Button
          style="outline"
          size="sm"
          disabled={refind.isPending}
          onclick={() => refind.mutate()}
        >
          Check again
        </Button>
      {/if}
      <Button
        style="outline"
        size="sm"
        onclick={() => (showConnectModal = true)}
      >
        Change account…
      </Button>
      <Button
        style="plain"
        size="sm"
        class="ml-auto text-red-400!"
        disabled={disconnect.isPending}
        onclick={() => disconnect.mutate(workspace)}
      >
        Disconnect
      </Button>
    </div>
  {:else}
    <p class="unconnected">
      Pull request and issue links still open in your browser, shown as
      repo#number. Connect this workspace to see whether they're merged,
      reviewed and green.
    </p>
  {/if}

  {#if ghStatus.isError}
    <ErrorBanner title="Could not ask the GitHub CLI" error={ghStatus.error} />
  {/if}
  {#if refind.isError}
    <ErrorBanner
      title="Could not ask the GitHub CLI"
      error={refind.error}
      onDismiss={() => refind.reset()}
    />
  {/if}
  {#if disconnect.isError}
    <ErrorBanner
      title="Could not disconnect from GitHub"
      error={disconnect.error}
      onDismiss={() => disconnect.reset()}
    />
  {/if}
</IntegrationCard>
<GitHubConnectModal bind:show={showConnectModal} {workspace} />

<style lang="postcss">
  @reference "tailwindcss";

  .path {
    @apply truncate font-mono text-xs/5;
  }
</style>
