<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import {
  activeWorkspaceQueryOptions,
  WorkspaceSettings,
  workspacesQueryOptions,
} from "#domains/workspaces";
import LoadingState from "#shared/ui/LoadingState.svelte";
import SidePanel from "#shell/SidePanel.svelte";

const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
const workspaces = createQuery(() => workspacesQueryOptions);
const hasWorkspaces = $derived((workspaces.data?.length ?? 0) > 0);
</script>

<main>
  {#if hasWorkspaces}
    <SidePanel />
  {/if}
  {#if activeWorkspace === undefined || workspaces.data === undefined}
    <LoadingState
      state={activeWorkspace.isError || workspaces.isError
        ? "error"
        : "loading"}
    >
      {#snippet loading()}
        Loading the Workspaces...
      {/snippet}
      {#snippet error()}
        Could not load the Workspaces.
      {/snippet}
    </LoadingState>
  {:else}
    <WorkspaceSettings
      activeWorkspace={activeWorkspace.data ?? null}
      workspaces={workspaces.data}
    />
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex;
  }
</style>
