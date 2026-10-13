<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { basename } from "#shared/utils/paths.ts";
  import type { Workspace } from "../../client/app";
  import { createSwitchWorkspaceMutation } from "../../client/mutations";
  import {
    activeWorkspaceQueryOptions,
    workspacesQueryOptions,
  } from "../../client/queries";
  import { toasts } from "#shared/utils/toasts.svelte.ts";
  import WorkspaceAvatar from "../common/WorkspaceAvatar.svelte";
  import WorkspacePicker from "../common/WorkspacePicker.svelte";

  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const workspace = $derived(activeWorkspace.data ?? null);
  const workspaces = createQuery(() => workspacesQueryOptions);

  // Stays on the current page, which then shows the other workspace's data
  const switchWorkspace = createSwitchWorkspaceMutation();
  const switchTo = (picked: Workspace) => {
    if (picked.isActive || switchWorkspace.isPending) return;
    if (!picked.available) {
      toasts.show(`Can't open “${picked.name}”: its database is missing`, {
        tone: "error",
      });
      return;
    }
    switchWorkspace.mutate(picked, {
      onError: () =>
        toasts.show(`Could not switch to “${picked.name}”`, { tone: "error" }),
    });
  };
</script>

<!-- Without an active workspace, the link to create or pick one -->
{#if workspace && workspaces.data}
  <WorkspacePicker
    workspaces={workspaces.data}
    selected={workspace}
    onPicked={switchTo}
    class="w-full"
  />
{:else}
<a href="/settings/workspace" class="workspace" title={workspace?.path}>
  <WorkspaceAvatar name={workspace?.name} />
  <span class="details">
    <span class="name">
      <span class="sr-only">Workspace:</span>
      {workspace?.name ?? "todai"}
    </span>
    {#if !workspace}
      <span class="subtitle">No active workspace</span>
    {:else if !workspace.available}
      <span class="subtitle unavailable">Unavailable</span>
    {:else}
      <span class="subtitle">{basename(workspace.path)}</span>
    {/if}
  </span>
</a>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .workspace {
    /* Layout */
    @apply flex items-center gap-3 rounded-xl p-1.5;
    /* States */
    @apply transition-colors hover:bg-white/5;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500;
  }

  .details {
    @apply flex flex-col;
    .name {
      @apply truncate text-sm font-semibold text-white;
    }
    .subtitle {
      @apply truncate text-xs text-slate-400;
      &.unavailable {
        @apply text-red-400;
      }
    }
  }
</style>
