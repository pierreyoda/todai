<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconPullRequestDraftProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconPullRequestDraftProps = $props();
</script>

<!-- A branch not ready to merge yet: dotted where it would merge -->
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
  <path d="M4 5.25v5.5M12 9.5v-.5M12 6.5V6M12 3.5V3" />
</svg>
