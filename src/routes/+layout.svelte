<script lang="ts">
  import "../app.css";
  import { QueryClientProvider } from "@tanstack/svelte-query";
  import type { Snippet } from "svelte";

  import { queryClient } from "../client/queries";

  type LayoutProps = {
    children: Snippet;
  };

  let { children }: LayoutProps = $props();
</script>

<QueryClientProvider client={queryClient}>
  <div class="w-screen h-screen">
    {@render children()}
  </div>
</QueryClientProvider>
