<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconPullRequestProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconPullRequestProps = $props();
</script>

<!-- Two branches, the second one merging back into the first -->
<svg
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 16 16"
  stroke-width="1.5"
  stroke="currentColor"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  class={["size-4", extraClass]}
>
  <circle cx="4" cy="3.5" r="1.75" />
  <circle cx="4" cy="12.5" r="1.75" />
  <circle cx="12" cy="12.5" r="1.75" />
  <path d="M4 5.25v5.5M12 10.75V6.5a2 2 0 0 0-2-2H7.5M9 3 7.5 4.5 9 6" />
</svg>
