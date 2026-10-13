<script lang="ts">
import { untrack } from "svelte";

import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import Modal from "#shared/ui/Modal.svelte";
import {
  createAddWorkspaceMutation,
  createRenameWorkspaceMutation,
} from "../api/mutations";
import type { Workspace } from "../model/types";
import WorkspaceForm from "./WorkspaceForm.svelte";

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

const close = () => (show = false);
// Creating a workspace switches to it
const createWorkspace = createAddWorkspaceMutation({ onSuccess: close });
const updateWorkspace = createRenameWorkspaceMutation({ onSuccess: close });

const failed = $derived(existing ? updateWorkspace : createWorkspace);
// A failure is only shown until the modal closes. Untracked, so that resetting doesn't rerun the effect.
$effect(() => {
  if (!show) {
    untrack(() => {
      createWorkspace.reset();
      updateWorkspace.reset();
    });
  }
});
</script>

<Modal bind:show>
  {#snippet header()}
    {#if existing}
      Edit the Workspace
    {:else}
      Create a new Workspace
    {/if}
  {/snippet}
  {#if failed.isError}
    <div class="mb-4">
      <ErrorBanner
        title={existing
          ? "Could not rename the workspace"
          : "Could not create the workspace"}
        error={failed.error}
        onDismiss={() => failed.reset()}
      />
    </div>
  {/if}
  {#if existing}
    <WorkspaceForm
      data={{ name: existing.name, path: existing.path }}
      onSubmit={(data) => updateWorkspace.mutate({ workspace: existing, name: data.name })}
    />
  {:else}
    <WorkspaceForm onSubmit={(data) => createWorkspace.mutate(data)} />
  {/if}
</Modal>
