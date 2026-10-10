<script lang="ts">
  import {
    createMutation,
    createQuery,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import { save } from "@tauri-apps/plugin-dialog";
  import { untrack } from "svelte";

  import { goto } from "$app/navigation";
  import {
    invokeApiClient,
    type Workspace,
    type WorkspaceBackup,
  } from "../../../client/app";
  import {
    workspaceBackupsQueryOptions,
    workspaceKeys,
  } from "../../../client/queries";
  import {
    basename,
    dateToTodaiDate,
    formatFileSize,
    SHOW_IN_FILE_MANAGER,
  } from "../../../utils";
  import Button from "../../common/Button.svelte";
  import BackupsTable from "./BackupsTable.svelte";
  import BackupRestoreModal from "./BackupRestoreModal.svelte";
  import BackupRestoreAsNewModal from "./BackupRestoreAsNewModal.svelte";
  import AutoBackupSettings from "./AutoBackupSettings.svelte";
  import ErrorBanner from "../../common/ErrorBanner.svelte";
  import SettingsContainer from "../SettingsContainer.svelte";
  import BackupsWorkspacePicker from "./BackupsWorkspacePicker.svelte";
  import IconArchiveBox from "../../common/icons/IconArchiveBox.svelte";
  import IconFolderOpen from "../../common/icons/IconFolderOpen.svelte";
  import IconArrowDownTray from "../../common/icons/IconArrowDownTray.svelte";

  type BackupsSettingsProps = {
    workspaces: readonly Workspace[];
    /** The one whose backups are shown, among `workspaces`. */
    workspace: Workspace;
  };
  const { workspaces, workspace }: BackupsSettingsProps = $props();

  const backups = createQuery(() => workspaceBackupsQueryOptions(workspace.id));
  const totalSize = $derived(
    (backups.data ?? []).reduce((total, { size }) => total + (size ?? 0), 0),
  );

  const queryClient = useQueryClient();
  // Also refreshes their workspace's last backup, in the list
  const refreshBackups = () =>
    queryClient.invalidateQueries({ queryKey: workspaceKeys.all });

  const createBackup = createMutation(() => ({
    mutationFn: ({ id }: Workspace) =>
      invokeApiClient({
        name: "create_workspace_backup",
        args: { workspaceId: id },
      }),
    onSuccess: refreshBackups,
  }));
  const deleteBackup = createMutation(() => ({
    mutationFn: ({ id }: WorkspaceBackup) =>
      invokeApiClient({ name: "delete_workspace_backup", args: { id } }),
    onSuccess: refreshBackups,
  }));
  const exportWorkspace = createMutation(() => ({
    mutationFn: ({ workspace, path }: { workspace: Workspace; path: string }) =>
      invokeApiClient({
        name: "export_workspace",
        args: { workspaceId: workspace.id, path },
      }),
  }));
  const showBackups = createMutation(() => ({
    mutationFn: ({ id }: Workspace) =>
      invokeApiClient({
        name: "open_workspace_backups_folder",
        args: { workspaceId: id },
      }),
  }));

  const mutations = [
    { mutation: createBackup, failure: "Could not back up the workspace" },
    { mutation: exportWorkspace, failure: "Could not export the workspace" },
    { mutation: deleteBackup, failure: "Could not delete the backup" },
    { mutation: showBackups, failure: "Could not open its backups folder" },
  ];
  // Outcomes are about the workspace they were for: dropped when picking another one. Untracked, so that resetting
  // doesn't rerun the effect.
  $effect(() => {
    void workspace.id;
    untrack(() => mutations.forEach(({ mutation }) => mutation.reset()));
  });

  let pickingExportPath = $state(false);
  const exportCopy = async () => {
    if (pickingExportPath || exportWorkspace.isPending) return;
    pickingExportPath = true;
    const path = await save({
      defaultPath: `${workspace.name} ${dateToTodaiDate(new Date())}.sqlite3`,
      filters: [{ name: "Todai Workspace", extensions: ["sqlite3"] }],
      canCreateDirectories: true,
    });
    pickingExportPath = false;
    if (!path) return;
    exportWorkspace.mutate({ workspace, path });
  };

  // Kept once closed, so that the modals don't lose their content while closing
  let showRestoreDialog = $state(false);
  let restoredBackup = $state<WorkspaceBackup | null>(null);
  let showRestoreAsNewDialog = $state(false);
  let restoredAsNewBackup = $state<WorkspaceBackup | null>(null);
</script>

<SettingsContainer>
  {#snippet title()}
    Backups
  {/snippet}
  <div class="content">
    <div class="toolbar">
      <BackupsWorkspacePicker
        {workspaces}
        selected={workspace}
        onPicked={({ id }) =>
          // Keeps the focus on the picker
          goto(`?workspace=${id}`, { replace: true, reset: false })}
      />
      <p class="summary" role="status">
        {#if exportWorkspace.isSuccess}
          Exported to {basename(exportWorkspace.variables.path)}
        {:else if backups.data?.length === 0}
          No backups
        {:else if backups.data}
          {backups.data.length}
          {backups.data.length === 1 ? "backup" : "backups"} · {formatFileSize(
            totalSize,
          )}
        {/if}
      </p>
      <Button
        style="plain"
        disabled={showBackups.isPending}
        onclick={() => showBackups.mutate(workspace)}
      >
        <span data-slot="icon"><IconFolderOpen class="size-full" /></span>
        {SHOW_IN_FILE_MANAGER}
      </Button>
      <Button
        style="outline"
        disabled={!workspace.available ||
          pickingExportPath ||
          exportWorkspace.isPending}
        onclick={exportCopy}
      >
        <span data-slot="icon"><IconArrowDownTray class="size-full" /></span>
        Export a copy…
      </Button>
      <Button
        disabled={!workspace.available || createBackup.isPending}
        onclick={() => createBackup.mutate(workspace)}
      >
        <span data-slot="icon"><IconArchiveBox class="size-full" /></span>
        Back up now
      </Button>
    </div>

    {#if !workspace.available}
      <ErrorBanner
        title="Its database is unavailable"
        error={`Nothing usable at ${workspace.path}: it can't be backed up or exported until it is.`}
      />
    {/if}
    {#each mutations as { mutation, failure } (failure)}
      {#if mutation.isError}
        <ErrorBanner
          title={failure}
          error={mutation.error}
          onDismiss={() => mutation.reset()}
        />
      {/if}
    {/each}

    {#if backups.isError}
      <ErrorBanner title="Could not list its backups" error={backups.error} />
    {:else if backups.data}
      <BackupsTable
        backups={backups.data}
        workspaceName={workspace.name}
        autoBackup={workspace.autoBackup}
        onRestore={(backup) => {
          restoredBackup = backup;
          showRestoreDialog = true;
        }}
        onRestoreAsNew={(backup) => {
          restoredAsNewBackup = backup;
          showRestoreAsNewDialog = true;
        }}
        onDelete={(backup) => deleteBackup.mutate(backup)}
      />
    {:else}
      <p class="loading" role="status">Loading its backups…</p>
    {/if}
    <!-- Recreated for each workspace, as it edits a copy of its settings -->
    {#key workspace.id}
      <AutoBackupSettings {workspace} />
    {/key}
  </div>
</SettingsContainer>
{#if restoredBackup}
  <BackupRestoreModal
    bind:show={showRestoreDialog}
    {workspace}
    backup={restoredBackup}
  />
{/if}
<!-- Recreated for each backup, as the form only reads its initial data -->
{#key restoredAsNewBackup}
  {#if restoredAsNewBackup}
    <BackupRestoreAsNewModal
      bind:show={showRestoreAsNewDialog}
      {workspace}
      backup={restoredAsNewBackup}
      onRestored={({ id }) => goto(`?workspace=${id}`, { replace: true })}
    />
  {/if}
{/key}

<style lang="postcss">
  @reference "tailwindcss";

  .content {
    @apply flex min-h-0 flex-1 flex-col gap-4 pt-4;
  }

  .toolbar {
    @apply flex flex-wrap items-center gap-3;
  }

  /* Pushes the actions to the end */
  .summary {
    @apply ml-auto text-xs text-slate-400 tabular-nums;
  }

  .loading {
    @apply px-2 py-1.5 text-sm text-slate-400;
  }
</style>
