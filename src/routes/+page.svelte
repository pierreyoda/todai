<script lang="ts">
  import TodoPanel from "../components/todos/TodoPanel.svelte";
  import { dateToTodaiDate } from "../utils/dates";

  const day = dateToTodaiDate(new Date());
</script>

<main>
  <TodoPanel {day} />
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex items-start;
  }
</style>
