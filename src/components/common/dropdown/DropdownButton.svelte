<script lang="ts">
  import type { ComponentProps } from "svelte";

  import Button from "../Button.svelte";
  import DropdownTrigger from "./DropdownTrigger.svelte";

  /** Same props as `Button`: clicking it toggles the menu. */
  type DropdownButtonProps = Omit<
    ComponentProps<typeof Button>,
    "onclick" | "ref"
  >;
  const props: DropdownButtonProps = $props();
</script>

<DropdownTrigger>
  {#snippet children(trigger)}
    <Button size="xs" {...props} {...trigger} />
  {/snippet}
</DropdownTrigger>
