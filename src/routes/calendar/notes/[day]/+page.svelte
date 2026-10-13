<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";
  import { activeWorkspaceQueryOptions } from "#domains/workspaces";
  import { noteQueryOptions } from "#domains/notes";
  import SidePanel from "../../../../components/SidePanel.svelte";
  import { NotePanel } from "#domains/notes";
  import LoadingState from "#shared/ui/LoadingState.svelte";
  import type { PageProps } from "./$types";

  const { data }: PageProps = $props();
  const day = $derived(data.day);

  // As on the todos' page: without a usable active workspace, the user picks or creates one.
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const ready = $derived(activeWorkspace.data?.available === true);
  $effect(() => {
    if (activeWorkspace.isSuccess && !ready) {
      goto("/settings/workspace");
    }
  });

  const note = createQuery(() => ({
    ...noteQueryOptions(day),
    enabled: ready,
  }));
</script>

<main>
  {#if ready}
    <SidePanel />
    <!-- Kept once loaded, even if refetching it fails: the editor holds what's being written. Started over for
         another day's note, whose page is the same. -->
    {#if note.data !== undefined}
      {#key day}
        <NotePanel {day} note={note.data} />
      {/key}
    {:else}
      <LoadingState state={note.isError ? "error" : "loading"}>
        {#snippet loading()}
          Loading the note...
        {/snippet}
        {#snippet error()}
          Could not load this note.
        {/snippet}
      </LoadingState>
    {/if}
  {:else}
    <LoadingState state={activeWorkspace.isError ? "error" : "loading"}>
      {#snippet loading()}
        Loading the Workspaces...
      {/snippet}
      {#snippet error()}
        Could not load the Workspaces.
      {/snippet}
    </LoadingState>
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply flex h-full w-full;
  }
</style>
