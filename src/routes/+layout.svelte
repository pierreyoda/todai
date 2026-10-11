<script lang="ts">
  import "../app.css";
  import { QueryClientProvider } from "@tanstack/svelte-query";
  import type { Snippet } from "svelte";

  import { queryClient } from "../client/queries";
  import Toaster from "../components/common/Toaster.svelte";

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
