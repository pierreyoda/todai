<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconEllipsisVerticalProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconEllipsisVerticalProps = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 24 24"
  stroke-width={1.5}
  stroke="currentColor"
  class={["size-4", extraClass]}
>
  <path
    stroke-linecap="round"
    stroke-linejoin="round"
    d="M12 6.75a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5ZM12 12.75a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5ZM12 18.75a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5Z"
  />
</svg>
