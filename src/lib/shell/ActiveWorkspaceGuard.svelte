<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import type { Snippet } from "svelte";

  import { goto } from "$app/navigation";
  import { activeWorkspaceQueryOptions } from "#domains/workspaces";
  import LoadingState from "#shared/ui/LoadingState.svelte";

  /**
   * Shows `children` only with a usable active workspace, whose database they query. Without one (none yet, removed, or
   * its database could not be opened), the user picks or creates one first.
   */
  type ActiveWorkspaceGuardProps = {
    children: Snippet;
  };
  const { children }: ActiveWorkspaceGuardProps = $props();

  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const ready = $derived(activeWorkspace.data?.available === true);
  $effect(() => {
    if (activeWorkspace.isSuccess && !ready) {
      goto("/settings/workspace");
    }
  });
</script>

{#if ready}
  {@render children()}
{:else}
  <main>
    <LoadingState state={activeWorkspace.isError ? "error" : "loading"}>
      {#snippet loading()}
        Loading the Workspaces...
      {/snippet}
      {#snippet error()}
        Could not load the Workspaces.
      {/snippet}
    </LoadingState>
  </main>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply flex h-full w-full;
  }
</style>
