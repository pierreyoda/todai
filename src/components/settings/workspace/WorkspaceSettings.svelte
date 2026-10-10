<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";
  import { open } from "@tauri-apps/plugin-dialog";

  import { invokeApiClient, type Workspace } from "../../../client/app";
  import {
    clearActiveWorkspace,
    invalidateActiveWorkspace,
    workspaceKeys,
  } from "../../../client/queries";
  import { basename } from "../../../utils";
  import ErrorBanner from "../../common/ErrorBanner.svelte";
  import IconFolderOpen from "../../common/icons/IconFolderOpen.svelte";
  import IconPlus from "../../common/icons/IconPlus.svelte";
  import SettingsContainer from "../SettingsContainer.svelte";
  import WorkspacesTable from "./WorkspacesTable.svelte";
  import WorkspaceUpsertModal from "./WorkspaceUpsertModal.svelte";
  import WorkspaceRemoveModal from "./WorkspaceRemoveModal.svelte";

  type WorkspaceSettingsProps = {
    activeWorkspace: Workspace | null;
    workspaces: readonly Workspace[];
  };
  const { activeWorkspace, workspaces }: WorkspaceSettingsProps = $props();
  const onlyNew = $derived(!activeWorkspace || workspaces.length === 0);

  const queryClient = useQueryClient();
  // Importing a workspace switches to it. Named after its file, without the extension.
  const importWorkspaceMutation = createMutation(() => ({
    mutationFn: (path: string) =>
      invokeApiClient({
        name: "import_workspace",
        args: { name: basename(path).replace(/(?<=.)\.[^.]*$/, ""), path },
      }),
    onSuccess: () => invalidateActiveWorkspace(queryClient),
  }));
  // Unregisters a workspace, keeping its database, and its backups unless asked otherwise. Removing the active one
  // leaves none active: its todos and tags are dropped, and the user picks or creates another one here.
  const removeWorkspaceMutation = createMutation(() => ({
    mutationFn: ({
      workspace,
      deleteBackups,
    }: {
      workspace: Workspace;
      deleteBackups: boolean;
    }) =>
      invokeApiClient({
        name: "remove_workspace",
        args: { id: workspace.id, deleteBackups },
      }),
    onSuccess: (_, { workspace: { isActive } }) => {
      showRemoveDialog = false;
      return isActive
        ? clearActiveWorkspace(queryClient)
        : queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  }));

  let showUpsertDialog = $state(false);
  /** Created if `null`. */
  let editedWorkspace = $state<Workspace | null>(null);
  const openUpsertDialog = (workspace: Workspace | null) => {
    editedWorkspace = workspace;
    showUpsertDialog = true;
  };

  let importingWorkspace = $state(false);
  const importWorkspace = async () => {
    if (importingWorkspace || importWorkspaceMutation.isPending) return;
    importingWorkspace = true;
    const path = await open({
      filters: [{ name: "Todai Workspace", extensions: ["sqlite3"] }],
    });
    importingWorkspace = false;
    if (!path) return;
    importWorkspaceMutation.mutate(path);
  };

  let showRemoveDialog = $state(false);
  /** Kept once closed, so that the modal doesn't lose its content while closing. */
  let removedWorkspace = $state<Workspace | null>(null);
  const openRemoveDialog = (workspace: Workspace) => {
    removedWorkspace = workspace;
    showRemoveDialog = true;
  };
</script>

<SettingsContainer>
  {#snippet title()}
    {#if onlyNew}
      Create a Workspace
    {:else}
      Workspaces
    {/if}
  {/snippet}
  <div class="flex flex-col grow">
    <!-- Centered together -->
    <div class="choices">
      <button class="add-area" onclick={() => openUpsertDialog(null)}>
        <span class="add-icon">
          <IconPlus class="size-12" />
        </span>
        <h3>Create a new Workspace</h3>
      </button>
      <button
        class="import-area"
        title="Import an existing Workspace"
        aria-label="Import an existing Workspace"
        disabled={importingWorkspace || importWorkspaceMutation.isPending}
        onclick={importWorkspace}
      >
        <span class="import-icon">
          <IconFolderOpen class="size-6" />
        </span>
        Import
      </button>
      {#if importWorkspaceMutation.isError}
        <ErrorBanner
          title="Could not import the workspace"
          error={importWorkspaceMutation.error}
          onDismiss={() => importWorkspaceMutation.reset()}
        />
      {/if}
    </div>
  </div>
  {#if workspaces.length > 0}
    <WorkspacesTable
      {workspaces}
      onEdit={openUpsertDialog}
      onRemove={openRemoveDialog}
    />
  {/if}
</SettingsContainer>
{#if removedWorkspace}
  {@const workspace = removedWorkspace}
  <WorkspaceRemoveModal
    bind:show={showRemoveDialog}
    {workspace}
    onDelete={(deleteBackups) =>
      removeWorkspaceMutation.mutate({ workspace, deleteBackups })}
  />
{/if}
<!-- Recreated for each workspace, as the form only reads its initial data -->
{#key editedWorkspace}
  {#if editedWorkspace}
    <WorkspaceUpsertModal
      bind:show={showUpsertDialog}
      existing={editedWorkspace}
    />
  {:else}
    <WorkspaceUpsertModal bind:show={showUpsertDialog} />
  {/if}
{/key}

<style lang="postcss">
  @reference "tailwindcss";

  /* Fixed width: otherwise an error banner's long message would widen it, leaving the buttons on its left */
  .choices {
    @apply self-center my-auto flex w-72 flex-col gap-3;
  }

  .add-area {
    /* Layout */
    @apply flex w-72 flex-col items-center gap-4 px-8 py-10;
    /* Surface: barely lifted from the background */
    @apply rounded-2xl bg-white/3 ring-1 ring-white/10;
    /* Typography */
    @apply font-medium tracking-wide text-slate-300;
    /* Ring-based hover, animated */
    @apply cursor-pointer transition duration-200 ease-out motion-reduce:transition-none;
    &:hover {
      @apply -translate-y-0.5 bg-white/5 text-white shadow-lg shadow-pink-500/10 ring-2 ring-pink-500/60;
      @apply motion-reduce:translate-none;
    }
  }

  .add-icon {
    @apply rounded-full bg-pink-500/10 p-4 text-pink-400;
    @apply transition duration-200 ease-out motion-reduce:transition-none;
    .add-area:hover & {
      @apply rotate-90 bg-pink-500/20 text-pink-300;
      @apply motion-reduce:rotate-none;
    }
  }

  .import-area {
    /* Layout: as wide as the add area, but slim */
    @apply relative flex w-72 items-center gap-4 justify-center overflow-hidden py-3;
    /* Surface: barely lifted from the background */
    @apply rounded-2xl bg-white/3 ring-1 ring-white/10;
    /* Typography */
    @apply font-medium tracking-wide text-slate-300;
    /* Ring-based hover, animated */
    @apply cursor-pointer transition duration-200 ease-out focus-visible:outline-none motion-reduce:transition-none;
    /* Sheen, swept across on hover */
    @apply before:absolute before:inset-0 before:-translate-x-full before:bg-linear-to-r before:from-transparent before:via-blue-400/15 before:to-transparent;
    @apply before:transition-transform before:duration-700 before:ease-out motion-reduce:before:hidden;
    /* While picking the file, then importing it */
    @apply disabled:cursor-wait disabled:opacity-60;
    &:enabled:hover,
    &:focus-visible {
      @apply -translate-y-0.5 bg-white/5 shadow-lg shadow-blue-500/10 ring-2 ring-blue-500/60;
      @apply before:translate-x-full motion-reduce:translate-none;
    }
  }

  .import-icon {
    @apply rounded-full bg-blue-500/10 p-2.5 text-blue-400;
    @apply transition-colors duration-200 ease-out motion-reduce:transition-none;
    .import-area:enabled:hover &,
    .import-area:focus-visible & {
      @apply bg-blue-500/20 text-blue-300;
      animation: hop 450ms ease-out;
      @apply motion-reduce:animate-none;
    }
  }

  /* A small hop, as if the folder was opening */
  @keyframes hop {
    0%,
    100% {
      transform: translateY(0) scale(1);
    }
    40% {
      transform: translateY(-3px) scale(1.1);
    }
    70% {
      transform: translateY(1px) scale(0.98);
    }
  }
</style>
