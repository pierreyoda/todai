<script lang="ts">
  import { createMutation, useQueryClient } from "@tanstack/svelte-query";

  import Modal from "../../common/Modal.svelte";
  import { invokeApiClient, type Workspace } from "../../../client/app";
  import { invalidateActiveWorkspace, workspaceKeys } from "../../../client/queries";
  import WorkspaceForm, { type WorkspaceFormData } from "./WorkspaceForm.svelte";

  type WorkspaceUpsertModalProps = {
    /** Bindable. */
    show: boolean;
  } & (
    | {
        existing: Workspace;
      }
    | {
        existing?: never;
      }
  );
  let { show = $bindable(), existing }: WorkspaceUpsertModalProps = $props();

  const queryClient = useQueryClient();

  // Creating a workspace switches to it.
  const createWorkspace = createMutation(() => ({
    mutationFn: ({ name, path }: WorkspaceFormData) =>
      invokeApiClient({
        name: "create_workspace",
        args: { name, path },
      }),
    onSuccess: () => {
      show = false;
      return invalidateActiveWorkspace(queryClient);
    },
  }));
  // Only its name can change: there is no command to move its database. Invalidating the list also refreshes the
  // active workspace.
  const updateWorkspace = createMutation(() => ({
    mutationFn: ({ workspace, data }: { workspace: Workspace; data: WorkspaceFormData }) =>
      invokeApiClient({
        name: "rename_workspace",
        args: { id: workspace.id, name: data.name },
      }),
    onSuccess: () => {
      show = false;
      return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  }));
</script>

<div class="modal-container">
  <Modal bind:show>
    {#snippet header()}
      {#if existing}
        Edit the Workspace
      {:else}
        Create a new Workspace
      {/if}
    {/snippet}
    {#if existing}
      <WorkspaceForm
        data={{ name: existing.name, path: existing.path }}
        onSubmit={(data) => updateWorkspace.mutate({ workspace: existing, data })}
      />
    {:else}
      <WorkspaceForm onSubmit={(data) => createWorkspace.mutate(data)} />
    {/if}
  </Modal>
</div>
