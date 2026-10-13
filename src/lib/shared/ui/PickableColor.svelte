<script lang="ts">
  import Button from "./Button.svelte";

  type PickableColorProps = {
    label: string;
    /** Format: "#RRGGBB" */
    preset: string;
    selected: boolean;
    onPicked: () => void;
  };

  const { label, preset, selected, onPicked }: PickableColorProps = $props();
</script>

<Button
  type="button"
  style="plain"
  title={label}
  onclick={onPicked}
  class={["color-preset", selected && "selected"]}
  --preset-color={preset}
></Button>

<style lang="postcss">
  @reference "tailwindcss";

  /* Global since the class lands on Button's inner <button>; `!` overrides Button's base styles */
  :global(.color-preset) {
    @apply size-5! shrink-0 cursor-pointer rounded-full! border! border-black/10! p-0! bg-(--preset-color)! transition-transform;
    @apply hover:scale-110 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
    @apply dark:border-white/15!;
  }

  :global(.color-preset.selected) {
    @apply transition duration-400;
    @apply scale-110 ring-1 ring-pink-500 ring-offset-2 ring-offset-white dark:ring-offset-zinc-900;
  }
</style>
