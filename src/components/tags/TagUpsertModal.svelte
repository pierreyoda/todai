<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import Modal from "../common/Modal.svelte";
  import { invokeClient } from "../../client";
  import type { Tag } from "../../client/types";
  import TagForm from "./TagForm.svelte";
  import { tagKeys } from "../../client/queries";

  type TagUpsertModalProps = {
    show: boolean;
  } & (
    | {
        existingTag: Tag;
        mode?: "all" | "color";
      }
    | {
        existingTag?: never;
        mode?: "all";
      }
  );
  let {
    show = $bindable(),
    existingTag,
    mode = "all",
  }: TagUpsertModalProps = $props();

  const queryClient = useQueryClient();
  const createTag = createMutation(() => ({
    mutationFn: (args: Pick<Tag, "name" | "color">) =>
      invokeClient({
        name: "create_tag",
        args,
      }),
    onSuccess: () => {
      show = false;
      queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));
  const updateTag = createMutation(() => ({
    mutationFn: (
      args: Pick<Tag, "id"> & Partial<Pick<Tag, "name" | "color">>,
    ) =>
      invokeClient({
        name: "update_tag",
        args,
      }),
    onSuccess: () => {
      show = false;
      queryClient.invalidateQueries({ queryKey: tagKeys.all });
    },
  }));
  const deleteTag = createMutation(() => ({
    mutationFn: (args: Pick<Tag, "id">) =>
      invokeClient({
        name: "delete_tag",
        args,
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
      {#if existingTag}
        {#if mode === "color"}
          Edit {existingTag.name} color
        {:else}
          Edit {existingTag.name}
        {/if}
      {:else}
        Create a new tag
      {/if}
    {/snippet}
    <TagForm
      {mode}
      data={{ name: existingTag?.name ?? "", color: existingTag?.color ?? "" }}
      onSubmit={(name, color) => {
        if (existingTag) updateTag.mutate({ id: existingTag.id, name, color });
        else createTag.mutate({ name, color });
      }}
      onDelete={() => deleteTag.mutate({ id: existingTag!.id })}
    />
  </Modal>
</div>
