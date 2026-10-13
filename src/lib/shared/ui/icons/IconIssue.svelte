<script lang="ts">
import type { ClassValue } from "svelte/elements";

type IconIssueProps = {
  class?: ClassValue;
};
const { class: extraClass }: IconIssueProps = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 16 16"
  stroke-width="1.5"
  stroke="currentColor"
  aria-hidden="true"
  class={["size-4", extraClass]}
>
  <circle cx="8" cy="8" r="6.25" />
  <circle cx="8" cy="8" r="1.25" fill="currentColor" />
</svg>
