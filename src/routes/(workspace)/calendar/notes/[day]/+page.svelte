<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { NotePanel, noteQueryOptions } from "#domains/notes";
import LoadingState from "#shared/ui/LoadingState.svelte";
import SidePanel from "#shell/SidePanel.svelte";
import type { PageProps } from "./$types";

const { data }: PageProps = $props();
const day = $derived(data.day);

const note = createQuery(() => noteQueryOptions(day));
</script>

<main>
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
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply flex h-full w-full;
  }
</style>
