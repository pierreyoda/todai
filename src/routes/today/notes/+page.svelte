<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";
  import {
    activeWorkspaceQueryOptions,
    noteQueryOptions,
  } from "../../../client/queries";
  import { dateToTodaiDate } from "../../../utils/dates";
  import SidePanel from "../../../components/SidePanel.svelte";
  import NotePanel from "../../../components/notes/NotePanel.svelte";
  import LoadingState from "../../../components/common/LoadingState.svelte";

  // As on the todos' page: without a usable active workspace, the user picks or creates one.
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const ready = $derived(activeWorkspace.data?.available === true);
  $effect(() => {
    if (activeWorkspace.isSuccess && !ready) {
      goto("/settings/workspace");
    }
  });

  // Fixed once opened, like the todos' day: left open past midnight, it's still the note of the day it was opened.
  const day = dateToTodaiDate(new Date());
  const note = createQuery(() => ({
    ...noteQueryOptions(day),
    enabled: ready,
  }));
</script>

<main>
  {#if ready}
    <SidePanel />
    <!-- Kept once loaded, even if refetching it fails: the editor holds what's being written -->
    {#if note.data !== undefined}
      <NotePanel {day} note={note.data} />
    {:else}
      <LoadingState state={note.isError ? "error" : "loading"}>
        {#snippet loading()}
          Loading today’s note...
        {/snippet}
        {#snippet error()}
          Could not load today’s note.
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
