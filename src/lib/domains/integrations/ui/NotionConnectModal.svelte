<script lang="ts">
import { untrack } from "svelte";
import { openLink } from "#domains/todos";
import type { Workspace } from "#domains/workspaces";
import Button from "#shared/ui/Button.svelte";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import FieldText from "#shared/ui/FieldText.svelte";
import Modal from "#shared/ui/Modal.svelte";
import { createConnectNotionMutation } from "../api/mutations";

type NotionConnectModalProps = {
  /** Bindable. */
  show: boolean;
  workspace: Workspace;
};
let { show = $bindable(), workspace }: NotionConnectModalProps = $props();

/** Where personal access tokens are created. */
const DEVELOPER_PORTAL_URL = "https://www.notion.so/profile/integrations";

const formId = $props.id();
const noteId = `${formId}-note`;
const connect = createConnectNotionMutation();

let input = $state<HTMLInputElement>();
let token = $state("");
let revealed = $state(false);
/** Replacing the token of a workspace already connected, rather than connecting it. */
const replacing = $derived(!!workspace.notion);
const accountName = $derived(
  workspace.notion?.name ?? workspace.notion?.email ?? "its Notion user",
);

// Blank each time it's opened, ready to paste into
$effect(() => {
  if (!show) return;
  untrack(() => {
    token = "";
    revealed = false;
    connect.reset();
  });
  // Once the dialog is shown, which would otherwise focus its close button
  setTimeout(() => input?.focus());
});

/** Why Notion didn't vouch for the token last tried, if so. */
const refusal = $derived(
  connect.data?.kind === "not_connected" ? connect.data.problem : undefined,
);
const canSubmit = $derived(token.trim() !== "" && !connect.isPending);

const onsubmit = (event: SubmitEvent) => {
  event.preventDefault();
  if (!canSubmit) return;
  connect.mutate(
    { workspace, token },
    {
      onSuccess: (outcome) => {
        if (outcome.kind === "connected") show = false;
      },
    },
  );
};
</script>

<Modal bind:show>
  {#snippet header()}
    {replacing
      ? `Replace Notion's token for ${workspace.name}`
      : `Connect Notion to ${workspace.name}`}
  {/snippet}
  <div class="body">
    {#if replacing}
      <p class="intro">
        {workspace.name} is connected as <span class="text-white">{accountName}</span>.
        Paste a new personal access token, e.g. once the current one expires: it
        replaces the one in your Keychain only once Notion accepts it. To use
        another Notion account, paste one of its tokens.
      </p>
    {:else}
      <p class="intro">
        For this workspace only, with a personal access token of yours: todai
        sees what you see in that Notion workspace, and only ever reads it. The
        token is kept in your macOS Keychain, never in the workspace or its
        backups.
      </p>
      <div class="setup">
        <p class="setup-title">
          In Notion's
          <a
            href={DEVELOPER_PORTAL_URL}
            onclick={(event) => {
              event.preventDefault();
              openLink(DEVELOPER_PORTAL_URL);
            }}>Developer portal</a
          >, under Personal access tokens, select New token:
        </p>
        <ol class="steps">
          <li>Name it, e.g. <strong>todai · {workspace.name}</strong></li>
          <li>Capability: <strong>Notion API</strong></li>
          <li>Workspace: the one this workspace's links point to</li>
          <li>
            Expiration: as you like. When Notion stops accepting it, todai tells
            you here
          </li>
          <li>Create it, then copy it: Notion shows it only once</li>
        </ol>
        <p class="setup-note">
          No Personal access tokens section? On Free and Business plans only
          owners can create them, unless an owner allows members in Settings ›
          Connections. Guests can't.
        </p>
      </div>
    {/if}

    <form id={formId} {onsubmit}>
      <FieldText
        label="Personal access token"
        type={revealed ? "text" : "password"}
        placeholder="Paste the token Notion showed you"
        autocomplete="off"
        spellcheck={false}
        class="token"
        aria-invalid={refusal?.kind === "token_refused"}
        aria-describedby={noteId}
        bind:value={token}
        bind:ref={input}
      >
        {#snippet trailing()}
          <Button
            style="plain"
            size="xs"
            class="reveal"
            aria-pressed={revealed}
            onclick={() => (revealed = !revealed)}
          >
            {revealed ? "Hide" : "Show"}
          </Button>
        {/snippet}
      </FieldText>
      {#if refusal?.kind === "token_refused"}
        <p id={noteId} class="note refused" role="alert">
          Notion refused this token: it may be cut short, expired or revoked.
          Copy it again, or create a new one.
        </p>
      {:else if refusal?.kind === "offline"}
        <p id={noteId} class="note warning" role="alert">
          Notion could not be reached, so the token wasn't checked nor kept: try
          again once online.
        </p>
      {:else if refusal}
        <p id={noteId} class="note warning" role="alert">
          Notion couldn't vouch for the token, so it wasn't kept: {refusal.kind ===
          "failed"
            ? refusal.message
            : `${refusal.kind.replace(/_/g, " ")}.`}
        </p>
      {:else}
        <p id={noteId} class="note">
          todai asks Notion who it belongs to before keeping it.
        </p>
      {/if}
    </form>

    {#if connect.isError}
      <ErrorBanner
        title="Could not keep the token"
        error={connect.error}
        onDismiss={() => connect.reset()}
      />
    {/if}
  </div>
  {#snippet actions()}
    <Button style="plain" onclick={() => (show = false)}>Cancel</Button>
    <Button type="submit" form={formId} disabled={!canSubmit}>
      {#if connect.isPending}
        Checking with Notion…
      {:else if refusal}
        Try again
      {:else if replacing}
        Replace token
      {:else}
        Connect
      {/if}
    </Button>
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

  .setup {
    @apply flex flex-col gap-2 rounded-xl bg-white/3 p-3 ring-1 ring-white/10;
  }

  .setup-title {
    @apply text-sm/5 font-medium text-white;

    a {
      @apply text-pink-400 underline underline-offset-2 hover:text-pink-300;
    }
  }

  .steps {
    @apply flex list-inside list-decimal flex-col gap-1 text-[13px]/5 text-slate-300;

    strong {
      @apply font-semibold text-white;
    }
  }

  .setup-note {
    @apply text-xs/[18px] text-slate-400;
  }

  form {
    @apply flex flex-col gap-1.5;
  }

  /* The field's input: a token reads better in a monospace font */
  form :global(.token input) {
    @apply font-mono text-[13px]/6;
  }

  /* A `Button`, restyled through its container as it's another component's element */
  form :global(.reveal) {
    @apply rounded-md border-0 px-2 py-0.5 text-xs/6 font-semibold text-slate-300;
  }

  .note {
    @apply text-xs/[18px] text-slate-400;

    &.refused {
      @apply text-[13px]/5 text-red-300;
    }
    &.warning {
      @apply text-[13px]/5 text-amber-300;
    }
  }
</style>
