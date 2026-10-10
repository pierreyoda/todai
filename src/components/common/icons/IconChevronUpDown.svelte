<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconChevronUpDownProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconChevronUpDownProps = $props();
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
    d="M8.25 15 12 18.75 15.75 15m-7.5-6L12 5.25 15.75 9"
  />
</svg>
