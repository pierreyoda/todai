<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import Modal from "../common/Modal.svelte";
  import { invokeClient } from "../../client";
  import type { Tag } from "../../client/types";
  import TagForm from "./TagForm.svelte";
  import { tagKeys } from "../../client/queries";

  type TagUpsertModalProps = {
    show: boolean;
    existingTag?: Tag;
  };
  let { show = $bindable(), existingTag }: TagUpsertModalProps = $props();

  const queryClient = useQueryClient();
  const createTag = createMutation(() => ({
    mutationFn: (newTag: Pick<Tag, "name" | "color">) =>
      invokeClient({
        name: "create_tag",
        args: newTag,
      }),
    onSuccess: () => {
      show = false;
      queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      Create a new tag
    {/snippet}
    <TagForm
      data={{ name: existingTag?.name ?? "", color: existingTag?.color ?? "" }}
      onSubmit={(name, color) => createTag.mutate({ name, color })}
    />
  </Modal>
</div>
