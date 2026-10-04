<script lang="ts">
  import { RGB_TAG_COLOR_PRESETS } from "../../constants";
  import Button from "../common/Button.svelte";
  import ConfirmButton from "../common/ConfirmButton.svelte";
  import FieldText from "../common/FieldText.svelte";
  import PickableColor from "../common/PickableColor.svelte";

  type TagFormProps = (
    | {
        /** Editing an existing tag, possibly only its color. */
        data: {
          name: string;
          color: string;
        };
        mode: "all" | "color";
        onDelete: () => void;
      }
    | {
        /** Creating a tag: needs its name too. */
        data?: never;
        mode: "all";
        onDelete?: never;
      }
  ) & {
    onSubmit: (name: string, color: string) => void;
  };
  const { data, mode, onSubmit, onDelete }: TagFormProps = $props();

  let editedName = $state(data?.name ?? "");
  let editedColor = $state(data?.color ?? null);
  const valid = $derived(editedName.trim().length > 0 && !!editedColor);
</script>

<div class="container">
  {#if mode === "all"}
    <FieldText bind:value={editedName} label="Name" />
  {/if}
  <div class="color-picker">
    {#each RGB_TAG_COLOR_PRESETS as { label, color } (color)}
      <PickableColor
        {label}
        preset={color}
        selected={editedColor === color}
        onPicked={() => (editedColor = color)}
      />
    {/each}
  </div>
  <Button
    type="submit"
    disabled={!valid}
    onclick={() => {
      if (!valid) return;
      onSubmit(editedName, editedColor!);
    }}
  >
    {#if data}
      Update
    {:else}
      Create
    {/if}
  </Button>
  {#if data}
    <ConfirmButton color="red" onclick={() => onDelete?.()}>Delete</ConfirmButton>
  {/if}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex flex-col gap-4 w-1/2 mx-auto;
  }

  .color-picker {
    @apply grid grid-cols-4 gap-4 py-4;
  }
</style>
