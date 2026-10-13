<script lang="ts">
import type { Snippet } from "svelte";

import ActiveWorkspaceGuard from "#shell/ActiveWorkspaceGuard.svelte";

type LayoutProps = {
  children: Snippet;
};

let { children }: LayoutProps = $props();
</script>

<!-- The pages of the active workspace's data: shown once it's usable -->
<ActiveWorkspaceGuard>
  {@render children()}
</ActiveWorkspaceGuard>
