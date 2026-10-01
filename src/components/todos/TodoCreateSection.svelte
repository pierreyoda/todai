<script lang="ts">
  import { invalidateAll } from "$app/navigation";

  import { invokeClient } from "../../client";
  import { dateToTodaiDate } from "../../utils/dates";
  import FieldText from "../inputs/FieldText.svelte";

  let title = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);

  const onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (submitting || !title.trim()) {
      return;
    }
    submitting = true;
    error = null;
    try {
      await invokeClient({
        name: "create_todo",
        args: { day: dateToTodaiDate(new Date()), title },
      });
      title = "";
      // Re-runs the page's `load`, to list the new todo.
      await invalidateAll();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
    }
  };
</script>

<form class="container" {onsubmit}>
  <FieldText
    label="New todo"
    hideLabel
    placeholder="Add a todo…"
    autocomplete="off"
    bind:value={title}
  />
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply w-full flex flex-col gap-1 p-2;
  }

  .error {
    @apply text-sm text-red-700;
  }
</style>
