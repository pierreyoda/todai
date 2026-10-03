<script lang="ts">
  import { RGB_TAG_COLOR_PRESETS } from "../../constants";
  import Button from "../common/Button.svelte";
  import FieldText from "../common/FieldText.svelte";
  import PickableColor from "../common/PickableColor.svelte";

  type TagFormProps = (
    | {
        data: {
          name: string;
          color: string;
        };
        mode: "all" | "color";
      }
    | {
        data?: never;
        mode: "color";
      }
  ) & {
    onSubmit: (name: string, color: string) => void;
    onDelete: () => void;
  };
  const { data, mode, onSubmit, onDelete }: TagFormProps = $props();

  let editedName = $state(data?.name ?? "");
  let editedColor = $state(data?.color ?? null);
  const valid = $derived(editedName.trim().length > 0 && !!editedColor);
  let confirmDelete = $state(false);
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
    {#if confirmDelete}
      Create
    {:else}
      Update
    {/if}
  </Button>
  <Button
    color="red"
    onclick={() => {
      if (confirmDelete) {
        onDelete();
        confirmDelete = false;
      } else confirmDelete = true;
    }}
  >
    {#if confirmDelete}
      Confirm
    {:else}
      Delete
    {/if}
  </Button>
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
