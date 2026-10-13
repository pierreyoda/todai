<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconXMarkProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconXMarkProps = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 24 24"
  stroke-width="1.5"
  stroke="currentColor"
  class={["size-4", extraClass]}
>
  <path
    stroke-linecap="round"
    stroke-linejoin="round"
    d="M6 18 18 6M6 6l12 12"
  />
</svg>
