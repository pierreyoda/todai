<script lang="ts">
  import Modal from "#shared/ui/Modal.svelte";
  import type { Tag } from "../model/types";
  import {
    createAddTagMutation,
    createDeleteTagMutation,
    createUpdateTagMutation,
  } from "../api/mutations";
  import TagForm from "./TagForm.svelte";

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

  const close = () => (show = false);
  const createTag = createAddTagMutation({ onSuccess: close });
  const updateTag = createUpdateTagMutation({ onSuccess: close });
  const deleteTag = createDeleteTagMutation({ onSuccess: close });
</script>

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
  {#if existingTag}
    <TagForm
      {mode}
      data={{ name: existingTag.name, color: existingTag.color }}
      onSubmit={(name, color) =>
        updateTag.mutate({ id: existingTag.id, name, color })}
      onDelete={() => deleteTag.mutate(existingTag.id)}
    />
  {:else}
    <TagForm
      mode="all"
      onSubmit={(name, color) => createTag.mutate({ name, color })}
    />
  {/if}
</Modal>
