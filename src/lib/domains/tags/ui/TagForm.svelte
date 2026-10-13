<script lang="ts">
import Button from "#shared/ui/Button.svelte";
import ConfirmButton from "#shared/ui/ConfirmButton.svelte";
import FieldText from "#shared/ui/FieldText.svelte";
import PickableColor from "#shared/ui/PickableColor.svelte";
import { RGB_TAG_COLOR_PRESETS } from "../model/colors";

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

// Follow the tag (once saved, e.g. renamed from the side panel while this form stays mounted), edited here until then
let editedName = $derived(data?.name ?? "");
let editedColor = $derived(data?.color ?? null);
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
