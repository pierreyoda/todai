<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import type { Tag } from "../../client/types";
  import { dateToTodaiDate } from "../../utils/dates";
  import { tagsQueryOptions } from "../../client/queries";
  import SidePanel from "../../components/SidePanel.svelte";
  import VerticalCalendar from "../../components/common/calendar/VerticalCalendar.svelte";

  const day = dateToTodaiDate(new Date());
  const tags = createQuery(() => tagsQueryOptions);
  let selectedTagId = $state<Tag["id"] | null>(null);
</script>

<main>
  <SidePanel
    tags={tags.isLoading ? "loading" : tags.error ? "error" : (tags.data ?? [])}
    {selectedTagId}
    onSelectedTagChanged={(tagId) => (selectedTagId = tagId)}
  />
  {#if tags.isLoading}
    Loading...
  {:else if tags.error}
    Error
  {:else}
    <section class="calendar">
      <VerticalCalendar tags={tags.data ?? []} {selectedTagId} today={day} />
    </section>
  {/if}
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }

  /* Takes the width left by the side panel, and scrolls on its own so the side panel stays in place */
  .calendar {
    @apply h-full min-w-0 flex-1 overflow-y-auto p-4;
  }
</style>
