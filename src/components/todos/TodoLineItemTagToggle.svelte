<script lang="ts">
  import type { UUID } from "node:crypto";
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import { invokeClient } from "../../client";
  import { tagKeys, todoKeys } from "../../client/queries";
  import type { Tag } from "../../client/types";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";
  import { dateToTodaiDate } from "../../utils";

  type TodoLineItemTagToggleProps = {
    todoId: UUID;
    todoTagsIds: readonly UUID[];
    tag: Tag;
  };
  const { todoId, todoTagsIds, tag }: TodoLineItemTagToggleProps = $props();

  let assigned = $derived(!!todoTagsIds.find((id) => id === tag.id));
  const queryClient = useQueryClient();
  const toggleTodoTag = createMutation(() => ({
    mutationFn: (tagIds: readonly UUID[]) =>
      invokeClient({
        name: "set_todo_tags",
        args: { todoId, tagIds },
      }),
    onSuccess: () => {
      // The todo's tags, and the tags' linked todos counts
      queryClient.invalidateQueries({
        queryKey: todoKeys.day(dateToTodaiDate(new Date())),
      });
      queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));
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
      toggleTodoTag.mutate(tagsIds);
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
