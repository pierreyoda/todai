<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import { createAddTodoLinkMutation } from "../../client/mutations";
  import { parseLinkQueryOptions } from "../../client/queries";
  import type { Todo } from "../../client/types";
  import { hasLink, linkKindName } from "../../utils/links";
  import Button from "../common/Button.svelte";
  import ErrorBanner from "../common/ErrorBanner.svelte";
  import FieldText from "../common/FieldText.svelte";
  import Modal from "../common/Modal.svelte";
  import LinkChip from "./LinkChip.svelte";
  import LinkKindIcon from "./LinkKindIcon.svelte";

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

  const addLink = createAddTodoLinkMutation(() => todo);

  // Blank each time it's opened, ready to paste into
  $effect(() => {
    if (!show) return;
    untrack(() => {
      url = "";
      typedUrl = "";
      addLink.reset();
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
  const recognized = $derived(settled && parsed.isSuccess ? parsed.data : undefined);
  const invalid = $derived(settled && parsed.isError ? String(parsed.error) : undefined);
  const alreadyLinked = $derived(!!recognized && hasLink(todo.links, recognized.url));
  // Not waiting for the recognition to settle: the backend checks the URL anyway
  const canAdd = $derived(url.trim() !== "" && !invalid && !alreadyLinked && !addLink.isPending);

  const onsubmit = (event: SubmitEvent) => {
    event.preventDefault();
    if (!canAdd) return;
    addLink.mutate(recognized?.url ?? url.trim(), {
      onSuccess: () => (show = false),
    });
  };
</script>

<div class="modal-container">
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
      {#if recognized}
        <div class="recognized" role="status">
          <span class="kind-icon"><LinkKindIcon kind={recognized.kind} /></span>
          <div class="details">
            <p class="kind">{linkKindName(recognized.kind)}</p>
            {#if recognized.github}
              <p class="name">
                {recognized.github.owner}/{recognized.github.repo}
                <span class="number">#{recognized.github.number}</span>
              </p>
            {:else}
              <p class="name url" title={recognized.url}>{recognized.url}</p>
            {/if}
            {#if alreadyLinked}
              <p class="note warning">Already on this todo.</p>
            {:else}
              <p class="note">
                {#if !recognized.github}
                  Not a GitHub pull request or issue: shown by its domain,
                {:else}
                  Shown on the todo as
                {/if}
                <LinkChip link={recognized} />
              </p>
            {/if}
          </div>
        </div>
      {:else if invalid}
        <p class="invalid" role="status">{invalid}</p>
      {/if}
      {#if addLink.isError}
        <ErrorBanner
          title="Could not add the link"
          error={addLink.error}
          onDismiss={() => addLink.reset()}
        />
      {/if}
    </form>
    {#snippet actions()}
      <Button style="plain" onclick={() => (show = false)}>Cancel</Button>
      <Button type="submit" form={formId} disabled={!canAdd}>Add link</Button>
    {/snippet}
  </Modal>
</div>

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
  }

  .kind-icon {
    @apply mt-0.5 shrink-0;

    > :global(svg) {
      @apply size-4.5 text-slate-400;
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

  .note {
    @apply mt-1 flex flex-wrap items-center gap-1.5 text-xs/5 text-slate-400;

    &.warning {
      @apply text-amber-300;
    }
  }

  .invalid {
    @apply text-sm/5 text-red-400;
  }
</style>
