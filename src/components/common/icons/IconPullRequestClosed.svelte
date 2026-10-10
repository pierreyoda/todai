<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  type IconPullRequestClosedProps = {
    class?: ClassValue;
  };
  const { class: extraClass }: IconPullRequestClosedProps = $props();
</script>

<!-- A branch crossed out where it would merge -->
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
  <path d="M4 5.25v5.5M12 10.75V8M10.5 2.5l3 3M13.5 2.5l-3 3" />
</svg>
