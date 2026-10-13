<script lang="ts">
import type { ClassValue } from "svelte/elements";

type IconIssueCompletedProps = {
  class?: ClassValue;
};
const { class: extraClass }: IconIssueCompletedProps = $props();
</script>

<!-- An issue, checked -->
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
  <circle cx="8" cy="8" r="6.25" />
  <path d="M5.5 8.25 7.25 10 10.5 6" />
</svg>
