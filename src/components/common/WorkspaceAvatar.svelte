<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type WorkspaceAvatarProps = {
    /** Its initial is shown, todai's without any. */
    name?: string;
    size?: "sm" | "md";
    class?: ClassValue;
  };
  const {
    name,
    size = "md",
    class: extraClass,
  }: WorkspaceAvatarProps = $props();
</script>

<!-- Decorative: the workspace's name is always written next to it -->
<span class={["avatar", size, extraClass]} aria-hidden="true">
  {name?.charAt(0) ?? "t"}
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .avatar {
    /* Layout */
    @apply grid shrink-0 place-items-center;
    /* Typography */
    @apply font-semibold text-white uppercase;
    /* Surface: the accent color, with a light inner edge like the solid buttons */
    @apply bg-linear-to-br from-pink-500 to-fuchsia-600 ring-1 ring-white/20 ring-inset;

    &.md {
      @apply size-8 rounded-lg text-sm shadow-md shadow-pink-500/20;
    }
    /* Fits a line of text */
    &.sm {
      @apply size-5 rounded-md text-[0.6875rem]/none;
    }
  }
</style>
