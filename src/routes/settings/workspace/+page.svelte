<script lang="ts">
  import WorkspaceSettings from "../../../components/settings/workspace/WorkspaceSettings.svelte";
  import SidePanel from "../../../components/SidePanel.svelte";
</script>

<main>
  <SidePanel />
  <WorkspaceSettings />
</main>

<style lang="postcss">
  @reference "tailwindcss";

  main {
    @apply w-full h-full flex;
  }
</style>
