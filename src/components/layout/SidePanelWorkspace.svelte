<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";

  import { basename } from "../../utils";
  import { activeWorkspaceQueryOptions } from "../../client/queries";

  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const workspace = $derived(activeWorkspace.data ?? null);
</script>

<a href="/settings/workspace" class="workspace" title={workspace?.path}>
  <span class="avatar" aria-hidden="true">
    {workspace?.name.charAt(0) ?? "t"}
  </span>
  <span class="details">
    <span class="name">
      <span class="sr-only">Workspace:</span>
      {workspace?.name ?? "todai"}
    </span>
    {#if !workspace}
      <span class="subtitle">No active workspace</span>
    {:else if !workspace.available}
      <span class="subtitle unavailable">Unavailable</span>
    {:else}
      <span class="subtitle">{basename(workspace.path)}</span>
    {/if}
  </span>
</a>

<style lang="postcss">
  @reference "tailwindcss";

  .workspace {
    /* Layout */
    @apply flex items-center gap-3 rounded-xl p-1.5;
    /* States */
    @apply transition-colors hover:bg-white/5;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500;
  }

  .avatar {
    /* Layout */
    @apply grid size-8 shrink-0 place-items-center rounded-lg;
    /* Typography */
    @apply text-sm font-semibold text-white uppercase;
    /* Surface: the accent color, with a light inner edge like the solid buttons */
    @apply bg-linear-to-br from-pink-500 to-fuchsia-600 shadow-md shadow-pink-500/20 ring-1 ring-white/20 ring-inset;
  }

  .details {
    @apply flex flex-col;
    .name {
      @apply truncate text-sm font-semibold text-white;
    }
    .subtitle {
      @apply truncate text-xs text-slate-400;
      &.unavailable {
        @apply text-red-400;
      }
    }
  }
</style>
