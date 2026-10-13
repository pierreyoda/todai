<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { dateToTodaiDate } from "#core/time";
import { NotePanel, noteQueryOptions } from "#domains/notes";
import LoadingState from "#shared/ui/LoadingState.svelte";
import SidePanel from "#shell/SidePanel.svelte";

// Fixed once opened, like the todos' day: left open past midnight, it's still the note of the day it was opened.
const day = dateToTodaiDate(new Date());
const note = createQuery(() => noteQueryOptions(day));
</script>

<main>
  <SidePanel />
  <!-- Kept once loaded, even if refetching it fails: the editor holds what's being written -->
  {#if note.data !== undefined}
    <NotePanel {day} note={note.data} />
  {:else}
    <LoadingState state={note.isError ? "error" : "loading"}>
      {#snippet loading()}
        Loading today's note...
      {/snippet}
      {#snippet error()}
        Could not load today's note.
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
