<script lang="ts">
  import "../app.css";
  import { QueryClientProvider } from "@tanstack/svelte-query";
  import type { Snippet } from "svelte";

  import { queryClient } from "#shared/ipc/queryClient.ts";
  import Toaster from "#shared/ui/Toaster.svelte";

  type LayoutProps = {
    children: Snippet;
  };

  let { children }: LayoutProps = $props();
</script>

<QueryClientProvider client={queryClient}>
  <div class="w-screen h-screen">
    {@render children()}
  </div>
  <Toaster />
</QueryClientProvider>
