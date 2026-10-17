<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { describeProblem, NotionDoneRules } from "#domains/todos";
import type { Workspace } from "#domains/workspaces";
import Button from "#shared/ui/Button.svelte";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import IconDocument from "#shared/ui/icons/IconDocument.svelte";
import { createDisconnectNotionMutation } from "../api/mutations";
import { notionApiStatusQueryOptions } from "../api/queries";
import { notionConnectionState } from "../model/connection";
import IntegrationCard from "./IntegrationCard.svelte";
import NotionConnectModal from "./NotionConnectModal.svelte";

type NotionCardProps = {
  workspace: Workspace;
};
const { workspace }: NotionCardProps = $props();

const disconnect = createDisconnectNotionMutation();
// Notion's side: whose token it is, which also tells whether Notion still accepts it
const apiStatus = createQuery(() => ({
  ...notionApiStatusQueryOptions(workspace.id),
  enabled: !!workspace.notion,
}));
const connection = $derived(
  notionConnectionState(
    workspace.notion,
    workspace.notionAttention,
    apiStatus.data,
  ),
);
const problem = $derived(apiStatus.data?.problem ?? workspace.notionAttention);
const apiProblem = $derived(problem && describeProblem("notion", problem));
/** Whose token it is: as Notion last said, else as recorded when connected. */
const account = $derived(apiStatus.data?.account ?? workspace.notion);
const accountLabel = $derived(
  account &&
    ([account.name, account.email].filter(Boolean).join(" · ") ||
      `Notion user ${account.userId}`),
);

let showConnectModal = $state(false);
</script>

<IntegrationCard
  name="Notion"
  subtitle="Pages and database items: status, assignees and due date"
  state={connection}
  onConnect={() => (showConnectModal = true)}
>
  {#snippet logo()}
    <IconDocument />
  {/snippet}

  {#if workspace.notion}
    <dl class="details">
      <dt>Account</dt>
      <dd>
        {accountLabel}
        {#if account?.workspaceName}
          <span class="text-slate-400">· {account.workspaceName}</span>
        {/if}
      </dd>
      <dt>Token</dt>
      <dd>Personal access token, kept in your macOS Keychain</dd>
      <dt>Notion</dt>
      <dd>
        {#if apiStatus.data?.account}
          Accepts the token
        {:else if apiProblem}
          <span class="text-amber-300">{apiProblem.label}</span>
        {:else if apiStatus.isError}
          <span class="text-slate-400">Unknown</span>
        {:else}
          <span class="text-slate-400">Asking Notion…</span>
        {/if}
      </dd>
    </dl>
    {#if problem?.kind === "token_refused"}
      <div class="problem" role="alert">
        <p class="problem-title">Notion refused this workspace's token</p>
        <p class="problem-text">
          It expired, or was revoked in Notion. Create a new personal access
          token in Notion's Developer portal, then paste it here. Until then,
          this workspace's Notion links show their last known status.
        </p>
      </div>
    {:else if problem?.kind === "token_missing"}
      <div class="problem" role="alert">
        <p class="problem-title">
          This workspace's token isn't in your Keychain anymore
        </p>
        <p class="problem-text">
          It was deleted from the Keychain, e.g. in Keychain Access. Paste it
          again, or a new one.
        </p>
      </div>
    {:else if apiProblem && problem?.kind !== "offline"}
      <div class="problem" role="alert">
        <p class="problem-title">Notion refused to answer</p>
        <p class="problem-text">{apiProblem.detail}</p>
      </div>
    {/if}
    <!-- Kept in the workspace's own database: only readable while it's the active one -->
    {#if workspace.isActive}
      <NotionDoneRules />
    {:else}
      <p class="unconnected text-xs/5!">
        Switch to {workspace.name} to see when its Notion pages are done.
      </p>
    {/if}
    <div class="actions">
      {#if connection === "attention"}
        <Button
          style="outline"
          size="sm"
          disabled={apiStatus.isFetching}
          onclick={() => apiStatus.refetch()}
        >
          Check again
        </Button>
      {/if}
      <Button
        style="outline"
        size="sm"
        onclick={() => (showConnectModal = true)}
      >
        {connection === "attention" ? "Paste a new token…" : "Replace token…"}
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
      Notion links still open in your browser, titled from their URL. Connect
      this workspace to see each page's status, assignees and due date.
    </p>
  {/if}

  {#if disconnect.isError}
    <ErrorBanner
      title="Could not disconnect from Notion"
      error={disconnect.error}
      onDismiss={() => disconnect.reset()}
    />
  {/if}
</IntegrationCard>
<NotionConnectModal bind:show={showConnectModal} {workspace} />
