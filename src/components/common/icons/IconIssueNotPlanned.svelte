<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconIssueNotPlannedProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconIssueNotPlannedProps = $props();
</script>

<!-- An issue, struck through -->
<svg
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 16 16"
  stroke-width="1.5"
  stroke="currentColor"
  stroke-linecap="round"
  aria-hidden="true"
  class={["size-4", extraClass]}
>
  <circle cx="8" cy="8" r="6.25" />
  <path d="M3.75 12.25l8.5-8.5" />
</svg>
