<script lang="ts">
  import { goto } from "$app/navigation";
  import type { Workspace } from "#domains/workspaces";
  import { WorkspacePicker } from "#domains/workspaces";
  import SettingsContainer from "#shared/ui/SettingsContainer.svelte";
  import GitHubCard from "./GitHubCard.svelte";
  import LinksBehaviorSettings from "./LinksBehaviorSettings.svelte";

  type IntegrationsSettingsProps = {
    workspaces: readonly Workspace[];
    /** The one whose integrations are shown, among `workspaces`. */
    workspace: Workspace;
  };
  const { workspaces, workspace }: IntegrationsSettingsProps = $props();
</script>

<SettingsContainer>
  {#snippet title()}
    Integrations
  {/snippet}
  <div class="content">
    <div class="toolbar">
      <WorkspacePicker
        {workspaces}
        selected={workspace}
        onPicked={({ id }) =>
          // Keeps the focus on the picker
          goto(`?workspace=${id}`, { replace: true, reset: false })}
      />
      <p class="hint">
        Each workspace has its own connections: one can use your own GitHub
        account, another your company's.
      </p>
    </div>
    <p class="intro">
      todai reads the pull requests and issues linked to this workspace's todos,
      to show where they stand. It never writes to GitHub, and stores no token:
      it asks the GitHub CLI for the connected account's when needed.
    </p>
    <!-- Recreated for each workspace, so that outcomes (e.g. a failed disconnection) don't carry over -->
    {#key workspace.id}
      <GitHubCard {workspace} />
      <LinksBehaviorSettings {workspace} />
    {/key}
  </div>
</SettingsContainer>

<style lang="postcss">
  @reference "tailwindcss";

  .content {
    @apply mx-auto flex w-full max-w-3xl flex-col gap-4 pt-4;
  }

  .toolbar {
    @apply flex flex-wrap items-center gap-3;
  }

  .hint {
    @apply min-w-0 flex-1 text-sm/5 text-slate-400;
  }

  .intro {
    @apply text-sm/6 text-slate-400;
  }
</style>
