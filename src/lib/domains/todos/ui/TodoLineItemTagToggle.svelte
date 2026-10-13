<script lang="ts">
  import type { UUID } from "node:crypto";

  import { createSetTodoTagsMutation } from "../api/mutations";
  import type { Day } from "#core/time";
  import type { Tag } from "#domains/tags";
  import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";

  type TodoLineItemTagToggleProps = {
    todoId: UUID;
    todoDay: Day;
    todoTagsIds: readonly UUID[];
    tag: Tag;
  };
  const { todoId, todoDay, todoTagsIds, tag }: TodoLineItemTagToggleProps =
    $props();

  let assigned = $derived(!!todoTagsIds.find((id) => id === tag.id));
  const toggleTodoTag = createSetTodoTagsMutation();
</script>

<div class="container">
  <FieldCheckbox
    label="Assigned?"
    hideLabel
    bind:checked={assigned}
    onchange={(event) => {
      const toAssign = event.currentTarget.checked;
      const tagsIds = toAssign
        ? [...new Set([...todoTagsIds, tag.id])]
        : todoTagsIds.filter((id) => id !== tag.id);
      toggleTodoTag.mutate({ todo: { id: todoId, day: todoDay }, tagIds: tagsIds });
    }}
  />
  <div class="flex items-center gap-2">
    <div class="rounded-xl w-4 h-4" style:background-color={tag.color}></div>
    <span class="text-white">{tag.name}</span>
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex items-center gap-4;
  }
</style>
