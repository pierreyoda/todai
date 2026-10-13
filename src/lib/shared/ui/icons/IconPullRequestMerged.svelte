<script lang="ts">
import type { ClassValue } from "svelte/elements";

type IconPullRequestMergedProps = {
  class?: ClassValue;
};
const { class: extraClass }: IconPullRequestMergedProps = $props();
</script>

<!-- A branch merged back into the first one -->
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
  <circle cx="12" cy="9" r="1.75" />
  <path d="M4 5.25v5.5M4 5.25c0 2.75 2.5 3.75 6.25 3.75" />
</svg>
