<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import type { Workspace } from "#domains/workspaces";
  import {
    createConnectGitHubMutation,
    createRefindGhMutation,
  } from "../api/mutations";
  import { ghStatusQueryOptions } from "../api/queries";
  import { preselectedGhAccount } from "../model/connection";
  import { openLink } from "#domains/todos";
  import Button from "#shared/ui/Button.svelte";
  import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
  import Modal from "#shared/ui/Modal.svelte";

  type GitHubConnectModalProps = {
    /** Bindable. */
    show: boolean;
    workspace: Workspace;
  };
  let { show = $bindable(), workspace }: GitHubConnectModalProps = $props();

  const formId = $props.id();
  const ghStatus = createQuery(() => ghStatusQueryOptions);
  const refind = createRefindGhMutation();
  const connect = createConnectGitHubMutation();

  const accounts = $derived(ghStatus.data?.accounts ?? []);
  const signedIn = $derived(accounts.filter((account) => account.signedIn));

  /** The login of the account to connect. */
  let picked = $state<string | null>(null);
  // Fresh each time it's opened: `gh auth login` may have been run since
  $effect(() => {
    if (!show) return;
    untrack(() => {
      connect.reset();
      refind.reset();
      ghStatus.refetch();
    });
  });
  // Preselected each time it's opened, once the accounts are known, and again if the picked one isn't available anymore
  $effect(() => {
    if (!show) {
      picked = null;
      return;
    }
    const preselected = preselectedGhAccount(accounts, workspace.githubLogin);
    untrack(() => {
      if (!signedIn.some((account) => account.login === picked)) {
        picked = preselected?.login ?? null;
      }
    });
  });

  const onsubmit = (event: SubmitEvent) => {
    event.preventDefault();
    if (!picked || connect.isPending) return;
    connect.mutate(
      { workspace, login: picked },
      { onSuccess: () => (show = false) },
    );
  };
</script>

<Modal bind:show>
  {#snippet header()}
    Connect GitHub to {workspace.name}
  {/snippet}
  <div class="body">
    <p class="intro">
      For this workspace only, through an account of the GitHub CLI (<code
        >gh</code
      >). todai only reads pull requests and issues, and stores no token: it
      asks <code>gh</code> for the account's when needed.
    </p>

    {#if ghStatus.data === undefined}
      {#if ghStatus.isError}
        <ErrorBanner
          title="Could not ask the GitHub CLI"
          error={ghStatus.error}
        />
      {:else}
        <p class="status" role="status">Looking for the GitHub CLI…</p>
      {/if}
    {:else if ghStatus.data.path === null}
      <div class="setup">
        <p class="setup-title">The GitHub CLI isn't installed</p>
        <ol class="steps">
          <li>
            Install it: <code>brew install gh</code>, or see
            <a
              href="https://cli.github.com"
              onclick={(event) => {
                event.preventDefault();
                openLink("https://cli.github.com");
              }}>cli.github.com</a
            >
          </li>
          <li>Sign in, in a terminal: <code>gh auth login</code></li>
          <li>Check again</li>
        </ol>
      </div>
    {:else if signedIn.length === 0}
      <div class="setup">
        <p class="setup-title">
          The GitHub CLI isn't signed in to any account on github.com
        </p>
        <ol class="steps">
          <li>Sign in, in a terminal: <code>gh auth login</code></li>
          <li>Check again</li>
        </ol>
      </div>
    {:else}
      <form id={formId} {onsubmit}>
        <fieldset class="accounts">
          <legend class="label">Account</legend>
          {#each accounts as account (account.login)}
            <label
              class={[
                "account",
                picked === account.login && "picked",
                !account.signedIn && "unavailable",
              ]}
            >
              <input
                type="radio"
                name="gh-account"
                value={account.login}
                disabled={!account.signedIn}
                bind:group={picked}
              />
              <span class="login">{account.login}</span>
              {#if !account.signedIn}
                <span class="tag warning">Signed out: run gh auth login</span>
              {:else if account.active}
                <span class="tag">Default in gh</span>
              {/if}
            </label>
          {/each}
        </fieldset>
      </form>
      <p class="path">GitHub CLI at {ghStatus.data.path}</p>
    {/if}

    {#if refind.isError}
      <ErrorBanner
        title="Could not ask the GitHub CLI"
        error={refind.error}
        onDismiss={() => refind.reset()}
      />
    {/if}
    {#if connect.isError}
      <ErrorBanner
        title="Could not connect to GitHub"
        error={connect.error}
        onDismiss={() => connect.reset()}
      />
    {/if}
  </div>
  {#snippet actions()}
    <Button style="plain" onclick={() => (show = false)}>Cancel</Button>
    {#if signedIn.length > 0}
      <Button
        type="submit"
        form={formId}
        disabled={!picked || connect.isPending}
      >
        Connect
      </Button>
    {:else}
      <Button
        disabled={refind.isPending || ghStatus.isFetching}
        onclick={() => refind.mutate()}
      >
        Check again
      </Button>
    {/if}
  {/snippet}
</Modal>

<style lang="postcss">
  @reference "tailwindcss";

  .body {
    @apply flex flex-col gap-4;
  }

  .intro {
    @apply text-sm/6 text-slate-400;
  }

  code {
    @apply rounded bg-white/5 px-1 py-0.5 font-mono text-xs text-slate-200;
  }

  .status {
    @apply text-sm/5 text-slate-400;
  }

  .setup {
    @apply flex flex-col gap-2 rounded-xl bg-white/3 p-3 ring-1 ring-white/10;
  }

  .setup-title {
    @apply text-sm/5 font-medium text-white;
  }

  .steps {
    @apply flex list-inside list-decimal flex-col gap-1 text-[13px]/5 text-slate-300;
    a {
      @apply font-medium text-pink-400 underline underline-offset-2 hover:text-pink-300;
    }
  }

  .accounts {
    @apply m-0 flex flex-col gap-2 border-0 p-0;
  }

  .label {
    @apply mb-1.5 p-0 text-sm/5 font-medium text-slate-300;
  }

  .account {
    @apply flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2.5 ring-1 ring-white/10 transition-colors hover:bg-white/5;

    &.picked {
      @apply bg-white/6 ring-pink-400/60;
    }
    &.unavailable {
      @apply cursor-not-allowed opacity-60 hover:bg-transparent;
    }

    input {
      @apply size-4 accent-pink-500;
    }
  }

  .login {
    @apply min-w-0 flex-1 truncate text-sm/5 font-medium text-white;
  }

  .tag {
    @apply shrink-0 rounded px-1.5 text-[11px]/[18px] font-semibold text-slate-300 ring-1 ring-white/15;

    &.warning {
      @apply text-amber-300 ring-amber-400/30;
    }
  }

  .path {
    @apply truncate font-mono text-xs/5 text-slate-500;
  }
</style>
