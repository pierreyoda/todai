<script lang="ts">
  import {
    createMutation,
    createQuery,
    useQueryClient,
  } from "@tanstack/svelte-query";

  import { invokeClient } from "../../client";
  import { createAddTodoFromLinkMutation } from "../../client/mutations";
  import {
    activeWorkspaceQueryOptions,
    invalidateTodosOf,
    linkPreviewQueryOptions,
    parseLinkQueryOptions,
  } from "../../client/queries";
  import type { Day } from "#core/time";
  import type { ParsedLink } from "../../client/types";
  import { mayBeLink } from "../../utils/links";
  import { itemState, linkStatus, stateStatus } from "../../utils/linkStatuses";
  import FieldText from "#shared/ui/FieldText.svelte";
  import IconDocumentText from "#shared/ui/icons/IconDocumentText.svelte";
  import LinkKindIcon from "../links/LinkKindIcon.svelte";

  type TodoCreateSectionProps = {
    day: Day;
  };
  const { day }: TodoCreateSectionProps = $props();

  /** How long creating a todo from a link waits for GitHub's answer, before titling it after the link instead. */
  const PREVIEW_WAIT_MS = 2000;

  const listboxId = $props.id();
  let title = $state("");
  const queryClient = useQueryClient();
  const createTodo = createMutation(() => ({
    mutationFn: (title: string) =>
      invokeClient({ name: "create_todo", args: { day, title } }),
    onSuccess: () => {
      title = "";
      // Returned so the mutation stays pending until the list is refetched.
      return invalidateTodosOf(queryClient, day);
    },
  }));
  const addFromLink = createAddTodoFromLinkMutation(() => day);

  // A pasted GitHub pull request or issue can become a todo titled after it, with its link, unless turned off
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const offered = $derived(!!activeWorkspace.data?.linksPasteToCreate);
  const typed = $derived(title.trim());
  const linkTyped = $derived(offered && mayBeLink(typed));
  const parsed = createQuery(() => ({
    ...parseLinkQueryOptions(typed),
    enabled: linkTyped,
  }));
  const recognized = $derived(
    linkTyped && parsed.data?.github ? parsed.data : undefined,
  );
  const connected = $derived(!!activeWorkspace.data?.githubLogin);
  const previewed = createQuery(() => ({
    ...linkPreviewQueryOptions(recognized?.url ?? ""),
    enabled: connected && !!recognized,
  }));
  const preview = $derived(
    connected && recognized && previewed.data?.link ? previewed.data : undefined,
  );
  const link = $derived(preview?.link ?? recognized);
  const snapshot = $derived(preview?.snapshot);
  const status = $derived(preview && linkStatus(preview));
  const lookingUp = $derived(connected && !!recognized && previewed.isPending);

  /** What was typed when the suggestions were dismissed (Escape), until it changes. */
  let dismissed = $state<string | null>(null);
  /** Waiting for GitHub's answer, once asked to create the todo. */
  let waiting = $state(false);
  const suggesting = $derived(!!link && dismissed !== typed);
  /** The suggestion picked by Enter. */
  let active = $state<"link" | "plain">("link");
  $effect(() => {
    if (!suggesting) active = "link";
  });
  const busy = $derived(createTodo.isPending || addFromLink.isPending || waiting);

  const createPlain = () => {
    if (busy || !typed) return;
    createTodo.mutate(title);
  };

  /** Creates a todo with the GitHub link `target`, titled after its pull request or issue if GitHub answers in time. */
  const createFromLink = async (target: ParsedLink) => {
    if (busy) return;
    const known = preview?.link.url === target.url || recognized?.url === target.url ? preview : undefined;
    let url = known?.link.url ?? target.url;
    let named = known?.snapshot?.title;
    if (!named && connected && !known) {
      // A moment for GitHub to answer, rather than titling the todo after the link's number
      waiting = true;
      const answer = await Promise.race([
        queryClient
          .fetchQuery(linkPreviewQueryOptions(target.url))
          .catch(() => undefined),
        new Promise<undefined>((resolve) =>
          setTimeout(() => resolve(undefined), PREVIEW_WAIT_MS),
        ),
      ]);
      waiting = false;
      named = answer?.snapshot?.title;
      url = answer?.link.url ?? url;
    }
    addFromLink.mutate(
      { url, title: named },
      { onSuccess: () => (title = "") },
    );
  };

  const onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (suggesting && link) {
      if (active === "link") createFromLink(link);
      else createPlain();
      return;
    }
    // Submitted right after pasting a link, before it's recognized: created from it, as it would have been suggested
    if (linkTyped && dismissed !== typed && !parsed.isFetched) {
      const target = await queryClient
        .fetchQuery(parseLinkQueryOptions(typed))
        .catch(() => undefined);
      if (target?.github) {
        createFromLink(target);
        return;
      }
    }
    createPlain();
  };

  const onkeydown = (event: KeyboardEvent) => {
    if (!suggesting) return;
    switch (event.key) {
      case "ArrowDown":
      case "ArrowUp":
        event.preventDefault();
        active = active === "link" ? "plain" : "link";
        break;
      case "Enter":
        event.preventDefault();
        if (event.shiftKey || active === "plain" || !link) createPlain();
        else createFromLink(link);
        break;
      case "Escape":
        event.preventDefault();
        dismissed = typed;
        break;
    }
  };
</script>

<form class="container" {onsubmit}>
  <FieldText
    label="New todo"
    hideLabel
    placeholder="Add a new todo item"
    autocomplete="off"
    role="combobox"
    aria-autocomplete="list"
    aria-expanded={suggesting}
    aria-controls={suggesting ? listboxId : undefined}
    aria-activedescendant={suggesting ? `${listboxId}-${active}` : undefined}
    bind:value={title}
    {onkeydown}
    onblur={() => (dismissed = typed)}
    onfocus={() => (dismissed = null)}
  />
  <!-- What the keys do, for screen readers: the suggestions only show it with key caps -->
  <span class="sr-only" role="status">
    {#if suggesting}
      Enter creates a todo from this link, Shift+Enter keeps the URL as its
      title.
    {/if}
  </span>
  {#if suggesting && link}
    <div
      id={listboxId}
      class="suggestions"
      role="listbox"
      aria-label="Create a todo from this link"
    >
      <!-- Picked with the keyboard from the field, as a combobox's options; clicked without taking the focus from it -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        id={`${listboxId}-link`}
        role="option"
        tabindex="-1"
        aria-selected={active === "link"}
        class={["option", active === "link" && "active"]}
        onpointerenter={() => (active = "link")}
        onmousedown={(event) => event.preventDefault()}
        onclick={() => createFromLink(link)}
      >
        <span class="option-icon">
          <LinkKindIcon
            kind={link.kind}
            state={snapshot && itemState(snapshot)}
          />
        </span>
        <div class="option-text">
          {#if snapshot}
            <p class="option-title">{snapshot.title}</p>
            <p class="option-detail">
              New todo, linked to <span class="label">{link.label}</span>
              {#if status}
                · <span class={["word", stateStatus(itemState(snapshot)).tone]}>
                  {stateStatus(itemState(snapshot)).word}
                </span>
                {#if status.word !== stateStatus(itemState(snapshot)).word}
                  · <span class={["word", status.tone]}>{status.word}</span>
                {/if}
              {/if}
            </p>
          {:else if lookingUp || waiting}
            <p class="option-title">
              Looking up {link.github?.owner}/{link.github?.repo}
              #{link.github?.number}…
            </p>
            <p class="option-detail">
              Press Enter now if you like: the todo is titled after it if
              GitHub answers in a moment, or “{link.label}” otherwise.
            </p>
          {:else}
            <p class="option-title">{link.label}</p>
            <p class="option-detail">
              New todo, linked to {link.label}.
              {#if !connected}
                This workspace isn't connected to GitHub, so the title comes
                from the URL: rename it as you like.
              {:else if preview?.fetchError}
                This workspace's GitHub account can't see it, so the title comes
                from the URL.
              {:else}
                GitHub couldn't be asked, so the title comes from the URL.
              {/if}
            </p>
          {/if}
        </div>
        <kbd aria-hidden="true">↵</kbd>
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        id={`${listboxId}-plain`}
        role="option"
        tabindex="-1"
        aria-selected={active === "plain"}
        class={["option", active === "plain" && "active"]}
        onpointerenter={() => (active = "plain")}
        onmousedown={(event) => event.preventDefault()}
        onclick={createPlain}
      >
        <span class="option-icon"><IconDocumentText /></span>
        <div class="option-text">
          <p class="option-title">Keep the URL as the title</p>
          <p class="option-detail">Plain text, no link attached</p>
        </div>
        <kbd aria-hidden="true">⇧↵</kbd>
      </div>
    </div>
  {/if}
  {#if createTodo.isError}
    <p class="error" role="alert">{String(createTodo.error)}</p>
  {/if}
  {#if addFromLink.isError}
    <p class="error" role="alert">{String(addFromLink.error)}</p>
  {/if}
</form>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply relative w-full flex flex-col gap-1 p-2;
  }

  .suggestions {
    /* Over the list, right under the field */
    @apply absolute top-full right-2 left-2 z-20 -mt-1 flex flex-col gap-0.5 p-1;
    @apply rounded-xl bg-slate-800/95 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;
  }

  .option {
    @apply flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2.5;

    &.active {
      @apply bg-white/8;
    }

    kbd {
      @apply shrink-0 rounded px-1.5 font-sans text-xs/5 text-slate-400 ring-1 ring-white/15;
    }
  }

  .option-icon {
    @apply shrink-0 self-start pt-0.5 text-slate-400;

    > :global(svg) {
      @apply size-4.5;
    }
  }

  .option-text {
    @apply flex min-w-0 flex-1 flex-col gap-0.5;
  }

  .option-title {
    @apply truncate text-sm/5 font-medium text-white;
  }

  .option-detail {
    @apply text-xs/5 text-slate-400;
  }

  .label {
    @apply font-medium text-slate-200 tabular-nums;
  }

  .word {
    @apply font-medium;

    &.open,
    &.good {
      @apply text-green-300;
    }
    &.bad {
      @apply text-red-300;
    }
    &.pending,
    &.warn {
      @apply text-amber-300;
    }
    &.merged {
      @apply text-purple-300;
    }
  }

  .error {
    @apply text-sm text-red-700;
  }
</style>
