<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import type { Workspace } from "../../../client/app";
  import {
    createDisconnectGitHubMutation,
    createRefindGhMutation,
  } from "../../../client/mutations";
  import {
    ghStatusQueryOptions,
    githubApiStatusQueryOptions,
  } from "../../../client/queries";
  import { formatTime } from "#shared/lib/timestamps.ts";
  import { githubConnectionState } from "../../../utils/github";
  import { describeProblem } from "../../../utils/linkStatuses";
  import Button from "#shared/ui/Button.svelte";
  import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
  import IconPullRequest from "#shared/ui/icons/IconPullRequest.svelte";
  import GitHubConnectModal from "./GitHubConnectModal.svelte";

  type GitHubCardProps = {
    workspace: Workspace;
  };
  const { workspace }: GitHubCardProps = $props();

  const titleId = $props.id();
  const ghStatus = createQuery(() => ghStatusQueryOptions);
  const refind = createRefindGhMutation();
  const disconnect = createDisconnectGitHubMutation();
  const connection = $derived(
    githubConnectionState(workspace.githubLogin, ghStatus.data),
  );
  const needsAttention = $derived(
    connection === "signed_out" || connection === "gh_missing",
  );
  // GitHub's side, once the GitHub CLI's is fine: which also tells whether GitHub accepts its token
  const apiStatus = createQuery(() => ({
    ...githubApiStatusQueryOptions(workspace.id),
    enabled: connection === "connected",
  }));
  const rateLimit = $derived(apiStatus.data?.rateLimit);
  const apiProblem = $derived(
    apiStatus.data?.problem && describeProblem(apiStatus.data.problem),
  );

  let showConnectModal = $state(false);
</script>

<article
  class={["card", needsAttention && "attention"]}
  aria-labelledby={titleId}
>
  <header class="header">
    <span class="logo"><IconPullRequest /></span>
    <div class="heading">
      <h3 id={titleId} class="name">GitHub</h3>
      <p class="subtitle">Pull requests and issues: state, checks and reviews</p>
    </div>
    {#if connection === "connected"}
      <span class="badge connected">Connected</span>
    {:else if needsAttention}
      <span class="badge warning">Needs attention</span>
    {:else if connection === "checking"}
      <span class="badge" role="status">Checking…</span>
    {:else}
      <Button onclick={() => (showConnectModal = true)}>Connect GitHub</Button>
    {/if}
  </header>

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
    <p class="subtitle">
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
</article>
<GitHubConnectModal bind:show={showConnectModal} {workspace} />

<style lang="postcss">
  @reference "tailwindcss";

  .card {
    @apply flex flex-col gap-3 rounded-xl bg-white/3 p-4 ring-1 ring-white/10;

    &.attention {
      @apply ring-amber-400/35;
    }
  }

  .header {
    @apply flex items-center gap-3;
  }

  .logo {
    @apply grid size-9 shrink-0 place-items-center rounded-lg bg-white/6 text-white ring-1 ring-white/10 ring-inset;

    > :global(svg) {
      @apply size-4.5;
    }
  }

  .heading {
    @apply min-w-0 flex-1;
  }

  .name {
    @apply text-[15px]/[22px] font-semibold text-white;
  }

  .subtitle {
    @apply text-xs/4 text-slate-400;

    .card > & {
      @apply text-sm/5;
    }
  }

  .badge {
    @apply inline-flex shrink-0 items-center gap-1.5 rounded-full bg-white/5 px-2.5 text-xs/5 font-medium text-slate-300;

    &::before {
      @apply size-1.5 rounded-full bg-current content-[''];
    }

    &.connected {
      @apply bg-green-500/12 text-green-300;
    }

    &.warning {
      @apply bg-amber-400/12 text-amber-300;
    }
  }

  .details {
    @apply grid grid-cols-[7.5rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-[13px]/5;

    dt {
      @apply text-slate-400;
    }

    dd {
      @apply text-white;
    }

    .path {
      @apply truncate font-mono text-xs/5;
    }
  }

  .problem {
    @apply flex flex-col gap-0.5 rounded-lg bg-amber-400/8 p-3 ring-1 ring-amber-400/30;
  }

  .problem-title {
    @apply text-sm/5 font-medium text-white;
  }

  .problem-text {
    @apply text-[13px]/5 text-slate-300;
  }

  code {
    @apply rounded bg-white/5 px-1 py-0.5 font-mono text-xs text-slate-200;
  }

  .actions {
    @apply flex items-center gap-2;
  }
</style>
