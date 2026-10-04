<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassValue } from "svelte/elements";

  import Button from "../common/Button.svelte";
  import IconXMark from "../common/icons/IconXMark.svelte";

  type TodoPanelAppliedFilterProps = {
    children: Snippet;
    onDismiss: () => void;
    class?: ClassValue;
  };
  const {
    children,
    onDismiss,
    class: extraClass,
  }: TodoPanelAppliedFilterProps = $props();
</script>

<div class={["container", extraClass]}>
  <Button
    style="plain"
    size="xs"
    class="text-gray-700 hover:text-gray-500"
    onclick={onDismiss}
  >
    <IconXMark />
  </Button>
  {@render children()}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex items-center gap-0.5 pr-2.5;
    @apply rounded-lg bg-gray-900;
  }
</style>
