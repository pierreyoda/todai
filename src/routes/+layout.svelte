<script lang="ts">
  import type { Snippet } from "svelte";

  type LayoutProps = {
    children: Snippet;
  };

  let { children }: LayoutProps = $props();
</script>

<div class="w-screen h-screen bg-slate-400">
  {@render children()}
</div>
