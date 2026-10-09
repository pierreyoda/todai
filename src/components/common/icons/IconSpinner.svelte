<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconSpinnerProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconSpinnerProps = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 24 24"
  stroke-width="2"
  stroke="currentColor"
  class={["size-4", extraClass]}
>
  <circle cx="12" cy="12" r="9" class="opacity-25" />
  <path stroke-linecap="round" d="M21 12a9 9 0 0 0-9-9" />
</svg>
