<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";

import { activeWorkspaceQueryOptions } from "#domains/workspaces";
import Button from "#shared/ui/Button.svelte";
import IconLink from "#shared/ui/icons/IconLink.svelte";
import IconXMark from "#shared/ui/icons/IconXMark.svelte";
import type { Todo } from "../../model/types";

type ConnectGitHubTipProps = {
  /** The todos shown: the tip is only for GitHub links. */
  todos: readonly Todo[];
};
const { todos }: ConnectGitHubTipProps = $props();

const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
const workspace = $derived(activeWorkspace.data);

/** Dismissed once per workspace, on this machine only. */
const storageKey = (workspaceId: string) =>
  `todai:connect-github-tip-dismissed:${workspaceId}`;
const readDismissed = (workspaceId: string) => {
  try {
    return localStorage.getItem(storageKey(workspaceId)) === "1";
  } catch {
    return false;
  }
};
/** The workspaces it was dismissed for since the page was opened. */
let dismissedNow = $state<string[]>([]);
const dismissed = $derived(
  !workspace ||
    dismissedNow.includes(workspace.id) ||
    readDismissed(workspace.id),
);

const shown = $derived(
  !!workspace &&
    !workspace.githubLogin &&
    !dismissed &&
    todos.some((todo) => todo.links.some((link) => link.github)),
);

const dismiss = () => {
  if (!workspace) return;
  dismissedNow = [...dismissedNow, workspace.id];
  try {
    localStorage.setItem(storageKey(workspace.id), "1");
  } catch {
    // Only for this time, then
  }
};
</script>

{#if shown}
  <aside class="tip" aria-label="Tip">
    <span class="tip-icon"><IconLink /></span>
    <div class="tip-text">
      <p class="tip-title">See checks, reviews and states on your links</p>
      <p class="tip-detail">
        Links work as they are: they open in your browser. Connect GitHub to
        this workspace and they also show where each pull request and issue
        stands.
      </p>
    </div>
    <a class="tip-action" href="/settings/integrations">Open Integrations</a>
    <Button
      style="plain"
      size="xs"
      class="shrink-0 text-slate-400! hover:text-white!"
      aria-label="Dismiss"
      title="Dismiss"
      onclick={dismiss}
    >
      <IconXMark />
    </Button>
  </aside>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .tip {
    @apply mx-2 mt-2 flex items-center gap-3 rounded-xl bg-white/3 px-3.5 py-3 ring-1 ring-white/10;
  }

  .tip-icon {
    @apply grid size-8 shrink-0 place-items-center rounded-full bg-pink-400/12 text-pink-400;

    > :global(svg) {
      @apply size-4;
    }
  }

  .tip-text {
    @apply min-w-0 flex-1;
  }

  .tip-title {
    @apply text-sm/5 font-medium text-white;
  }

  .tip-detail {
    @apply text-xs/5 text-slate-400;
  }

  .tip-action {
    @apply shrink-0 rounded-md bg-zinc-600 px-2.5 py-1.25 text-[13px]/5 font-semibold text-white shadow-[inset_0_1px_rgb(255_255_255/0.15)] transition-colors hover:bg-zinc-500;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }
</style>
