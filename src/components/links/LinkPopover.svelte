<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { untrack } from "svelte";

  import type { Todo, TodoLink } from "../../client/types";
  import {
    createRefreshTodoLinkMutation,
    createRemoveTodoLinkMutation,
  } from "../../client/mutations";
  import { activeWorkspaceQueryOptions } from "../../client/queries";
  import { formatRelativeTime } from "../../utils/dates";
  import { hasLink } from "../../utils/links";
  import {
    checkName,
    describeChecks,
    describeProblem,
    describeReview,
    itemState,
    linkStatus,
    notableChecks,
    stateStatus,
  } from "../../utils/linkStatuses";
  import { openLink } from "../../utils/openLink";
  import Button from "../common/Button.svelte";
  import ErrorBanner from "../common/ErrorBanner.svelte";
  import IconArrowPath from "../common/icons/IconArrowPath.svelte";
  import IconArrowTopRightOnSquare from "../common/icons/IconArrowTopRightOnSquare.svelte";
  import IconExclamationTriangle from "../common/icons/IconExclamationTriangle.svelte";
  import { computePosition } from "../common/dropdown/positioning";
  import LinkChip from "./LinkChip.svelte";
  import LinkKindIcon from "./LinkKindIcon.svelte";

  type LinkPopoverProps = {
    /** Bindable. */
    open: boolean;
    /** A GitHub link of `todo`. */
    link: TodoLink;
    todo: Todo;
    /** What it's shown next to: its chip. */
    anchor: HTMLElement | undefined;
  };
  let { open = $bindable(), link, todo, anchor }: LinkPopoverProps = $props();

  const titleId = $props.id();
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const refresh = createRefreshTodoLinkMutation(() => todo);
  const remove = createRemoveTodoLinkMutation(() => todo);

  const snapshot = $derived(link.snapshot);
  const linkState = $derived(snapshot && itemState(snapshot));
  const status = $derived(linkStatus(link));
  const github = $derived(link.github);
  const isIssue = $derived(
    snapshot ? snapshot.type === "issue" : link.kind === "github_issue",
  );
  const refreshProblem = $derived(
    refresh.data?.problem && describeProblem(refresh.data.problem),
  );

  const CHECK_OUTCOMES = {
    passed: "Passed",
    failed: "Failed",
    running: "Running",
    skipped: "Skipped",
  } as const;

  let popover = $state<HTMLDivElement>();

  const position = () => {
    if (!popover || !anchor) return;
    const { x, y } = computePosition({
      anchor: anchor.getBoundingClientRect(),
      floating: popover.getBoundingClientRect(),
      viewport: {
        width: document.documentElement.clientWidth,
        height: document.documentElement.clientHeight,
      },
      placement: "bottom-start",
      offset: 6,
    });
    popover.style.left = `${x}px`;
    popover.style.top = `${y}px`;
  };

  // Follows `open`: shown as a popover, so that clicking outside or pressing Escape closes it
  $effect(() => {
    if (!popover) return;
    const shown = popover.matches(":popover-open");
    if (open && !shown) {
      // What the last refresh ran into is only worth showing until closed
      untrack(() => refresh.reset());
      popover.showPopover({ source: anchor });
      // Positioned in the same task as the opening, so it never paints at the default position
      position();
      popover.focus({ preventScroll: true });
    } else if (!open && shown) {
      popover.hidePopover();
    }
  });

  // Kept next to its chip, which can move, e.g. when the list scrolls
  $effect(() => {
    if (!open) return;
    window.addEventListener("resize", position);
    window.addEventListener("scroll", position, {
      capture: true,
      passive: true,
    });
    return () => {
      window.removeEventListener("resize", position);
      window.removeEventListener("scroll", position, { capture: true });
    };
  });

  /** Whether focus was within when it closed: then given back to its chip. */
  let hadFocus = false;
</script>

<div
  bind:this={popover}
  popover="auto"
  role="dialog"
  tabindex="-1"
  aria-labelledby={titleId}
  class="popover"
  onbeforetoggle={(event) => {
    if (event.newState === "closed") {
      hadFocus = !!popover?.contains(document.activeElement);
    }
  }}
  ontoggle={(event) => {
    open = event.newState === "open";
    if (!open && hadFocus) anchor?.focus({ preventScroll: true });
  }}
>
  <header class="header">
    <div class="context">
      <LinkKindIcon kind={link.kind} state={linkState} />
      <span class="where">
        {isIssue ? "Issue" : "Pull request"}
        {#if github}
          · {github.owner}/{github.repo}
          <span class="number">#{github.number}</span>
        {/if}
      </span>
      <Button
        style="plain"
        size="xs"
        class="refresh"
        aria-label="Refresh from GitHub"
        title="Refresh from GitHub"
        disabled={refresh.isPending}
        onclick={() => refresh.mutate(link)}
      >
        <IconArrowPath
          class={[refresh.isPending && "motion-safe:animate-spin"]}
        />
      </Button>
    </div>
    <h2 id={titleId} class="title">{snapshot?.title ?? link.label}</h2>
    {#if snapshot && linkState}
      {@const statePill = stateStatus(linkState)}
      <div class="pills">
        <span class={["pill", statePill.tone]}>{statePill.word}</span>
        {#if status && status.word !== statePill.word}
          <span class={["pill", status.tone]}>{status.word}</span>
        {/if}
        {#if snapshot.type === "pull_request"}
          <span class="branches">{snapshot.headRef} → {snapshot.baseRef}</span>
        {:else}
          {#each snapshot.labels as label (label.name)}
            <span class="label">
              <span
                class="dot"
                style:background-color={label.color}
                aria-hidden="true"
              ></span>
              {label.name}
            </span>
          {/each}
        {/if}
      </div>
    {/if}
  </header>

  {#if link.fetchError}
    <p class="notice" role="alert">
      <IconExclamationTriangle class="shrink-0" />
      {#if link.fetchError === "sso_required"}
        Its organization requires single sign-on: authorize the GitHub CLI's
        token for it on github.com, then refresh.
      {:else}
        GitHub doesn't let {activeWorkspace.data?.githubLogin ?? "this account"}
        see it: it may be private, deleted, or not approved by its organization.
      {/if}
    </p>
  {/if}
  {#if refreshProblem}
    <p class="notice" role="alert">
      <IconExclamationTriangle class="shrink-0" />
      {refreshProblem.detail}
    </p>
  {/if}
  {#if refresh.isError}
    <div class="px-3 pb-2">
      <ErrorBanner
        title="Could not refresh it"
        error={refresh.error}
        onDismiss={() => refresh.reset()}
      />
    </div>
  {/if}
  {#if remove.isError}
    <div class="px-3 pb-2">
      <ErrorBanner
        title="Could not remove the link"
        error={remove.error}
        onDismiss={() => remove.reset()}
      />
    </div>
  {/if}

  {#if snapshot?.type === "pull_request"}
    <dl class="details">
      {#if snapshot.checks}
        <div class="row">
          <dt>Checks</dt>
          <dd class="checks">
            <span>{describeChecks(snapshot.checks)}</span>
            <span class="check-list">
              {#each notableChecks(snapshot.checks) as check, i (i)}
                <span class={["check", check.outcome]}>
                  {#if check.outcome === "passed"}
                    <svg
                      viewBox="0 0 16 16"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      aria-hidden="true"
                    >
                      <path d="M3.5 8.5 6.5 11.5 12.5 4.5" />
                    </svg>
                  {:else if check.outcome === "failed"}
                    <svg
                      viewBox="0 0 16 16"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      aria-hidden="true"
                    >
                      <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
                    </svg>
                  {:else}
                    <span class="ring" aria-hidden="true"></span>
                  {/if}
                  <!-- Told by its glyph, or else in words -->
                  <span class="sr-only">{CHECK_OUTCOMES[check.outcome]}:</span>
                  <span class="check-name">
                    {checkName(
                      check,
                    )}{#if check.outcome === "running" && check.startedAt},
                      started {formatRelativeTime(check.startedAt)}{/if}
                  </span>
                </span>
              {/each}
            </span>
          </dd>
        </div>
      {/if}
      <div class="row">
        <dt>Review</dt>
        <dd>{describeReview(snapshot)}</dd>
      </div>
      {#if snapshot.closes.length > 0}
        <div class="row">
          <dt>Closes</dt>
          <dd class="related">
            {#each snapshot.closes as item (item.url)}
              <span class="related-item">
                <LinkChip
                  link={item}
                  state={item.state}
                  status={stateStatus(item.state)}
                />
                {#if hasLink(todo.links, item.url)}
                  <span class="hint">Already on this todo</span>
                {/if}
              </span>
            {/each}
          </dd>
        </div>
      {/if}
      <div class="row">
        <dt>Changes</dt>
        <dd class="tabular-nums">
          <span class="text-green-300">
            +{snapshot.additions}<span class="sr-only"> lines added,</span>
          </span>
          <span class="text-red-300">
            −{snapshot.deletions}<span class="sr-only"> removed,</span>
          </span>
          · {snapshot.changedFiles}
          {snapshot.changedFiles === 1 ? "file" : "files"}
        </dd>
      </div>
      <div class="row">
        <dt>Updated</dt>
        <dd>
          {formatRelativeTime(snapshot.updatedAt)}
          {#if snapshot.author}by {snapshot.author}{/if}
        </dd>
      </div>
    </dl>
  {:else if snapshot?.type === "issue"}
    <dl class="details">
      {#if snapshot.closedBy.length > 0}
        <div class="row">
          <dt>Closed by</dt>
          <dd class="related">
            {#each snapshot.closedBy as item (item.url)}
              <span class="related-item">
                <LinkChip
                  link={item}
                  state={item.state}
                  status={stateStatus(item.state)}
                />
                {#if hasLink(todo.links, item.url)}
                  <span class="hint">Already on this todo</span>
                {/if}
              </span>
            {/each}
          </dd>
        </div>
      {/if}
      <div class="row">
        <dt>{snapshot.assignees.length > 1 ? "Assignees" : "Assignee"}</dt>
        <dd>{snapshot.assignees.join(", ") || "Nobody"}</dd>
      </div>
      {#if snapshot.milestone}
        <div class="row">
          <dt>Milestone</dt>
          <dd>{snapshot.milestone}</dd>
        </div>
      {/if}
      <div class="row">
        <dt>Updated</dt>
        <dd>
          {formatRelativeTime(snapshot.updatedAt)}
          {#if snapshot.author}· opened by {snapshot.author}{/if}
          · {snapshot.comments}
          {snapshot.comments === 1 ? "comment" : "comments"}
        </dd>
      </div>
    </dl>
  {/if}

  <footer class="footer">
    <!-- Announced as it's refreshed, then fetched -->
    <span class="fetched" role="status">
      {#if refresh.isPending}
        Refreshing…
      {:else if link.fetchedAt}
        Fetched {formatRelativeTime(link.fetchedAt).toLowerCase()}
      {:else}
        Not fetched yet
      {/if}
    </span>
    <Button
      style="plain"
      size="xs"
      class="remove"
      disabled={remove.isPending}
      onclick={() => remove.mutate(link, { onSuccess: () => (open = false) })}
    >
      Remove link
    </Button>
    <a
      class="open"
      href={link.url}
      onclick={(event) => {
        event.preventDefault();
        openLink(link.url);
      }}
    >
      Open on GitHub
      <IconArrowTopRightOnSquare class="size-3.5" />
    </a>
  </footer>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .popover {
    /* Reset popover defaults: positioned by script */
    @apply inset-auto m-0 border-0;
    /* Sizing: scrolls when taller than the viewport */
    @apply w-98 max-w-[calc(100vw-1rem)] max-h-[calc(100dvh-1rem)] overflow-y-auto overscroll-contain p-1;
    /* Surface */
    @apply rounded-xl bg-slate-800/92 text-sm/5 text-slate-300 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;
    @apply focus:outline-hidden;
  }

  .header {
    @apply flex flex-col gap-2.5 px-3 pt-3 pb-2.5;
  }

  .context {
    @apply flex items-center gap-2 text-slate-400;

    > :global(svg) {
      @apply size-4 shrink-0;
    }
  }

  .where {
    @apply min-w-0 flex-1 truncate text-xs/4;
  }

  .number {
    @apply tabular-nums;
  }

  /* A `Button`, restyled through its container as it's another component's element: more specific than its own rules */
  .context :global(.refresh) {
    @apply grid size-7 shrink-0 place-items-center rounded-md border-0 p-0 text-slate-400 transition-colors;
    @apply hover:text-white disabled:cursor-default disabled:opacity-100 focus-visible:outline-offset-0;

    /* Highlighted when hovered only, not when pressed */
    &:enabled:active {
      @apply bg-transparent;
    }
    &:enabled:hover {
      @apply hover:bg-white/10;
    }
  }

  .title {
    @apply text-[15px]/[22px] font-semibold text-white wrap-break-word;
  }

  .pills {
    @apply flex flex-wrap items-center gap-1.5;
  }

  .pill {
    @apply inline-flex items-center rounded-full bg-white/5 px-2 py-0.5 text-xs/4 font-semibold text-slate-300 ring-1 ring-white/15 ring-inset;

    &.open,
    &.good {
      @apply bg-green-500/15 text-green-300 ring-green-500/35;
    }
    &.bad {
      @apply bg-red-500/15 text-red-300 ring-red-500/35;
    }
    &.pending,
    &.warn {
      @apply bg-amber-400/12 text-amber-300 ring-amber-400/30;
    }
    &.merged {
      @apply bg-purple-500/15 text-purple-300 ring-purple-500/35;
    }
  }

  .branches {
    @apply min-w-0 truncate text-xs/4 text-slate-400;
  }

  .label {
    @apply inline-flex items-center gap-1 text-xs/4 text-slate-300;
  }

  .dot {
    @apply size-2 shrink-0 rounded-full;
  }

  .notice {
    @apply mx-3 mb-2 flex items-start gap-2 rounded-lg bg-amber-400/10 px-2.5 py-2 text-[13px]/5 text-amber-200;

    > :global(svg) {
      @apply mt-0.5 size-4;
    }
  }

  .details {
    @apply flex flex-col border-t border-white/6;
  }

  .row {
    @apply grid grid-cols-[5.5rem_minmax(0,1fr)] gap-3 px-3 py-2;

    & + & {
      @apply border-t border-white/4;
    }

    dt {
      @apply text-xs/5 text-slate-400;
    }

    dd {
      @apply text-[13px]/5 text-white;
    }
  }

  .checks {
    @apply flex flex-col gap-0.5;
  }

  /* Scrolls past about seven checks, the next one cut to show there's more */
  .check-list {
    @apply flex max-h-40 flex-col gap-0.5 overflow-y-auto overscroll-contain;
  }

  .check {
    @apply flex items-center gap-1.5 text-slate-300;

    > :global(svg) {
      @apply size-3 shrink-0;
    }
    &.passed > :global(svg) {
      @apply text-green-300;
    }
    &.failed > :global(svg) {
      @apply text-red-300;
    }
    .ring {
      @apply mx-0.5 size-2 shrink-0 rounded-full border-[1.5px] border-amber-300;
    }
  }

  .check-name {
    @apply min-w-0 truncate;
  }

  .related {
    @apply flex flex-col items-start gap-1;
  }

  .related-item {
    @apply flex items-center gap-2;
  }

  .hint {
    @apply text-xs/4 text-slate-400;
  }

  .footer {
    @apply flex items-center gap-2 border-t border-white/10 py-2 pr-2 pl-3;
  }

  .fetched {
    /* Dimmer than the rest, yet readable enough (WCAG AA) */
    @apply flex-1 text-xs/4 text-slate-400;
  }

  /* A `Button`, restyled as `.refresh` */
  .footer :global(.remove) {
    @apply rounded-md border-0 px-2 py-1 text-xs/4 font-medium text-red-400 transition-colors;
    @apply disabled:cursor-default disabled:opacity-50 focus-visible:outline-offset-0;

    /* Highlighted when hovered, even disabled, but not when pressed */
    &:enabled:active {
      @apply bg-transparent;
    }
    &:enabled:hover,
    &:disabled:hover {
      @apply hover:bg-red-500/10;
    }
  }

  .open {
    @apply inline-flex items-center gap-1.5 rounded-md bg-zinc-600 px-2.5 py-1.25 text-[13px]/[18px] font-semibold text-white shadow-[inset_0_1px_rgb(255_255_255/0.15)] transition-colors hover:bg-zinc-500;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }
</style>
