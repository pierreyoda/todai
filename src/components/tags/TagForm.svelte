<script lang="ts">
  import { RGB_TAG_COLOR_PRESETS } from "../../constants";
  import Button from "../common/Button.svelte";
  import FieldText from "../common/FieldText.svelte";
  import PickableColor from "../common/PickableColor.svelte";

  type TagFormProps = {
    data: {
      name: string;
      color: string;
    } | null;
    onSubmit: (name: string, color: string) => void;
  };
  const { data, onSubmit }: TagFormProps = $props();

  let editedName = $state(data?.name ?? "");
  let editedColor = $state(data?.color ?? RGB_TAG_COLOR_PRESETS[0].color);
</script>

<div class="container">
  <FieldText bind:value={editedName} label="Name" />
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
  <Button type="submit" onclick={() => onSubmit(editedName, editedColor)}>
    Create
  </Button>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex flex-col gap-4 w-1/2 mx-auto;
  }

  .color-picker {
    @apply grid grid-cols-4 gap-4;
  }
</style>
