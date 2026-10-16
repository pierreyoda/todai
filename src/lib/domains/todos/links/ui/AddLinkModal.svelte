<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import { providerInfo } from "#core/integrations";
import { activeWorkspaceQueryOptions, isConnected } from "#domains/workspaces";
import Button from "#shared/ui/Button.svelte";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";
import FieldText from "#shared/ui/FieldText.svelte";
import IconExclamationTriangle from "#shared/ui/icons/IconExclamationTriangle.svelte";
import Modal from "#shared/ui/Modal.svelte";
import { goto } from "$app/navigation";
import type { Todo } from "../../model/types";
import { createAddTodoLinksMutation } from "../api/mutations";
import { linkPreviewQueryOptions, parseLinkQueryOptions } from "../api/queries";
import { closingPullRequest, itemState } from "../model/github";
import { hasLink, linkKindName, linkProvider } from "../model/links";
import { describeProblem, linkStatusWords } from "../model/statuses";
import LinkChip from "./LinkChip.svelte";
import LinkKindIcon from "./LinkKindIcon.svelte";
import LinkStatusWord from "./LinkStatusWord.svelte";

type AddLinkModalProps = {
  /** Bindable. */
  show: boolean;
  todo: Todo;
};
let { show = $bindable(), todo }: AddLinkModalProps = $props();

const formId = $props.id();
let input = $state<HTMLInputElement>();
let url = $state("");
/** `url`, trimmed, once typing pauses: what's recognized. */
let typedUrl = $state("");
$effect(() => {
  const value = url.trim();
  const timeout = setTimeout(() => (typedUrl = value), 150);
  return () => clearTimeout(timeout);
});
/** Whether to also add the pull request closing the issue linked. */
let alsoAddClosing = $state(true);

const addLinks = createAddTodoLinksMutation(() => todo);
const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);

// Blank each time it's opened, ready to paste into
$effect(() => {
  if (!show) return;
  untrack(() => {
    url = "";
    typedUrl = "";
    alsoAddClosing = true;
    addLinks.reset();
  });
  // Once the dialog is shown, which would otherwise focus its close button
  setTimeout(() => input?.focus());
});

const parsed = createQuery(() => ({
  ...parseLinkQueryOptions(typedUrl),
  enabled: typedUrl !== "",
}));
/** Whether what's recognized is what's typed, rather than what was typed a moment ago. */
const settled = $derived(typedUrl !== "" && typedUrl === url.trim());
const recognized = $derived(
  settled && parsed.isSuccess ? parsed.data : undefined,
);
const invalid = $derived(
  settled && parsed.isError ? String(parsed.error) : undefined,
);

// What its provider tells about it, through the workspace's connection
const provider = $derived(recognized && linkProvider(recognized));
const connected = $derived(
  !!provider && isConnected(activeWorkspace.data, provider),
);
const previewed = createQuery(() => ({
  ...linkPreviewQueryOptions(recognized?.url ?? ""),
  enabled: connected,
}));
const preview = $derived(
  connected && previewed.data?.link ? previewed.data : undefined,
);
const lookingUp = $derived(connected && previewed.isFetching && !preview);
const info = $derived(provider && providerInfo(provider));
/** Who GitHub's notices tell the workspace is connected as. */
const githubLogin = $derived(activeWorkspace.data?.githubLogin);
/** Where its provider says it is, e.g. a pull request pasted as an issue. */
const link = $derived(preview?.link ?? recognized);
const snapshot = $derived(preview?.snapshot);
const words = $derived(preview ? linkStatusWords(preview) : []);
const problem = $derived(
  provider && preview?.problem && describeProblem(provider, preview.problem),
);

const alreadyLinked = $derived(
  !!recognized &&
    (hasLink(todo.links, recognized.url) ||
      (!!link && hasLink(todo.links, link.url))),
);
const closing = $derived(closingPullRequest(snapshot, todo.links));
const urls = $derived.by(() => {
  const added = link?.url ?? url.trim();
  return closing && alsoAddClosing ? [added, closing.url] : [added];
});
// Not waiting for the recognition to settle: the backend checks the URL anyway
const canAdd = $derived(
  url.trim() !== "" && !invalid && !alreadyLinked && !addLinks.isPending,
);

const onsubmit = (event: SubmitEvent) => {
  event.preventDefault();
  if (!canAdd) return;
  addLinks.mutate(urls, { onSuccess: () => (show = false) });
};
</script>

<Modal bind:show>
  {#snippet header()}
    Add a link
  {/snippet}
  <form id={formId} class="form" {onsubmit}>
    <p class="target">To “{todo.title}”</p>
    <FieldText
      label="URL"
      placeholder="A GitHub pull request or issue, or any link"
      autocomplete="off"
      spellcheck={false}
      aria-invalid={!!invalid}
      bind:value={url}
      bind:ref={input}
    />
    {#if link && preview?.fetchError}
      <div class="recognized warning" role="status">
        <span class="kind-icon"><IconExclamationTriangle /></span>
        <div class="details">
          <p class="kind">
            {linkKindName(link.kind)} · {link.github?.owner}/{link.github?.repo}
            <span class="number">#{link.github?.number}</span>
          </p>
          {#if preview.fetchError === "sso_required"}
            <p class="name">Its organization requires single sign-on</p>
            <p class="note">
              Authorize the GitHub CLI's token for {link.github?.owner} on github.com
              to see its status. You can still add it: it shows as {link.label},
              without a status.
            </p>
          {:else}
            <p class="name">This workspace's GitHub account can't see it</p>
            <p class="note">
              {activeWorkspace.data?.name} is connected as {githubLogin}, who can't
              see {link.github?.owner}/{link.github?.repo}: it may be private, or
              belong in another workspace. You can still add it here: it shows as
              {link.label}, without a status.
            </p>
          {/if}
          <a
            class="settings-link"
            href="/settings/integrations"
            onclick={(event) => {
              event.preventDefault();
              show = false;
              goto("/settings/integrations");
            }}
          >
            Check {activeWorkspace.data?.name}'s {info?.name} connection
          </a>
        </div>
      </div>
    {:else if link}
      <div class="recognized" role="status">
        <span class="kind-icon">
          <LinkKindIcon kind={link.kind} state={snapshot && itemState(snapshot)} />
        </span>
        <div class="details">
          <p class="kind">
            {linkKindName(link.kind)}
            {#if link.github}
              · {link.github.owner}/{link.github.repo}
              <span class="number">#{link.github.number}</span>
            {/if}
          </p>
          {#if snapshot}
            <p class="name">{snapshot.title}</p>
            <p class="facts">
              <!-- On one line: whitespace would show between the words -->
              {#each words as word, i (word.word)}{#if i > 0}{" · "}{/if}<LinkStatusWord status={word} look="word" />{/each}
              {#if snapshot.type === "issue"}
                {#each snapshot.labels as label (label.name)}
                  · {label.name}
                {/each}
                {#if snapshot.milestone}
                  · milestone {snapshot.milestone}
                {/if}
              {/if}
            </p>
          {:else if !link.github}
            <p class="name url" title={link.url}>{link.url}</p>
          {/if}
          {#if alreadyLinked}
            <p class="note warning">Already on this todo.</p>
          {:else if lookingUp}
            <p class="note">Looking it up {info?.where}…</p>
          {:else if !snapshot}
            <p class="note">
              {#if !link.github}
                Not a GitHub pull request or issue: shown by its domain,
              {:else}
                Shown on the todo as
              {/if}
              <LinkChip {link} />
            </p>
            {#if problem}
              <p class="note">
                Its status will show once {info?.name} can be asked:
                {problem.label.toLowerCase()}.
              </p>
            {/if}
          {/if}
        </div>
      </div>
      {#if closing && !alreadyLinked}
        <FieldCheckbox
          label={`Also add ${closing.label}, the pull request that closes it`}
          bind:checked={alsoAddClosing}
        />
      {/if}
    {:else if invalid}
      <p class="invalid" role="status">{invalid}</p>
    {/if}
    {#if addLinks.isError}
      <ErrorBanner
        title={urls.length > 1 ? "Could not add the links" : "Could not add the link"}
        error={addLinks.error}
        onDismiss={() => addLinks.reset()}
      />
    {/if}
  </form>
  {#snippet actions()}
    <Button style="plain" onclick={() => (show = false)}>Cancel</Button>
    <Button type="submit" form={formId} disabled={!canAdd}>
      {urls.length > 1 ? `Add ${urls.length} links` : "Add link"}
    </Button>
  {/snippet}
</Modal>

<style lang="postcss">
  @reference "tailwindcss";

  .form {
    @apply flex w-full flex-col gap-4;
  }

  .target {
    /* Closer to the modal's title, which it completes */
    @apply -mt-2 truncate text-sm/5 text-slate-400;
  }

  .recognized {
    @apply flex gap-3 rounded-xl bg-white/3 p-3 ring-1 ring-white/10;

    &.warning {
      @apply bg-amber-400/6 ring-amber-400/30;

      .kind-icon > :global(svg) {
        @apply text-amber-300;
      }
    }
  }

  .kind-icon {
    @apply mt-0.5 shrink-0 text-slate-400;

    > :global(svg) {
      @apply size-4.5;
    }
  }

  .details {
    @apply flex min-w-0 flex-col gap-0.5;
  }

  .kind {
    @apply text-xs/4 text-slate-400;
  }

  .name {
    @apply text-sm/5 font-semibold text-white;

    &.url {
      @apply truncate font-normal;
    }
  }

  .number {
    @apply tabular-nums;
  }

  .facts {
    @apply text-xs/5 text-slate-400;
  }

  .note {
    @apply mt-1 flex flex-wrap items-center gap-1.5 text-xs/5 text-slate-400;

    &.warning {
      @apply text-amber-300;
    }
  }

  .settings-link {
    @apply mt-1 self-start rounded-sm text-xs/5 font-medium text-pink-400 underline underline-offset-2 hover:text-pink-300;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }

  .invalid {
    @apply text-sm/5 text-red-400;
  }
</style>
