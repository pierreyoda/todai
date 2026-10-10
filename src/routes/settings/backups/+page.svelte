<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { page } from "$app/state";
  import { workspacesQueryOptions } from "../../../client/queries";
  import SidePanel from "../../../components/SidePanel.svelte";
  import LoadingState from "../../../components/common/LoadingState.svelte";
  import BackupsSettings from "../../../components/settings/backups/BackupsSettings.svelte";

  const workspaces = createQuery(() => workspacesQueryOptions);
  const hasWorkspaces = $derived((workspaces.data?.length ?? 0) > 0);

  // Picked in the URL (e.g. from the workspaces page), else the active one, else the first one. Even without a usable
  // active workspace: restoring one of its backups is how to get it back.
  const workspace = $derived.by(() => {
    const list = workspaces.data ?? [];
    const id = page.url.searchParams.get("workspace");
    return (
      list.find((workspace) => workspace.id === id) ??
      list.find((workspace) => workspace.isActive) ??
      list[0] ??
      null
    );
  });
</script>

<main>
  {#if hasWorkspaces}
    <SidePanel />
  {/if}
  {#if workspaces.data === undefined}
    <LoadingState state={workspaces.isError ? "error" : "loading"}>
      {#snippet loading()}
        Loading the Workspaces...
      {/snippet}
      {#snippet error()}
        Could not load the Workspaces.
      {/snippet}
    </LoadingState>
  {:else if workspace}
    <BackupsSettings workspaces={workspaces.data} {workspace} />
  {:else}
    <p class="no-workspaces">
      No workspaces to back up yet: <a href="/settings/workspace">create one</a>
      first.
    </p>
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex;
  }

  .no-workspaces {
    @apply m-auto text-sm text-slate-400;
    a {
      @apply font-medium text-pink-400 underline underline-offset-2 hover:text-pink-300;
    }
  }
</style>
