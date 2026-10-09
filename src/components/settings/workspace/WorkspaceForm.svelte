<script lang="ts" module>
  export type WorkspaceFormData = Pick<Workspace, "name" | "path">;
</script>

<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";

  import Button from "../../common/Button.svelte";
  import type { Workspace } from "../../../client/app";
  import FieldText from "../../common/FieldText.svelte";
  import { isValidPath } from "../../../utils";
  import IconMagnifyingGlass from "../../common/icons/IconMagnifyingGlass.svelte";

  type WorkspaceFormProps = (
    | {
        /** Editing an existing Workspace. */
        data: WorkspaceFormData;
      }
    | {
        /** Creating a new Workspace. */
        data?: never;
      }
  ) & {
    onSubmit: (submittedData: WorkspaceFormData) => void;
  };
  const { data, onSubmit }: WorkspaceFormProps = $props();

  let editedName = $state(data?.name ?? "");
  let editedPath = $state(data?.path ?? "");
  let editingPath = $state(false);
  const valid = $derived(
    editedName.trim().length > 0 && !editingPath && isValidPath(editedPath),
  );

  const pickFilePath = async () => {
    if (editedPath) return;
    editingPath = true;
    const picked = await save({
      defaultPath: `${editedName || "workspace"}.sqlite3`,
      filters: [{ name: "Todai Workspace", extensions: ["sqlite3"] }],
      canCreateDirectories: true,
    });
    editingPath = false;
    if (!picked) return;
    editedPath = picked;
  };
</script>

<div class="container">
  <FieldText bind:value={editedName} label="Name" />
  {#if !data}
    <!-- Bottom-aligned, so that the button lines up with the input rather than with its label -->
    <div class="flex items-end gap-2">
      <FieldText bind:value={editedPath} label="Database file path" />
      <Button
        size="xl"
        color="amber"
        disabled={editingPath}
        onclick={pickFilePath}
      >
        <IconMagnifyingGlass />
      </Button>
    </div>
  {/if}
  <Button
    type="submit"
    disabled={!valid}
    onclick={() => {
      if (!valid) return;
      onSubmit({
        name: editedName,
        path: editedPath,
      });
    }}
  >
    {#if data}
      Update
    {:else}
      Create
    {/if}
  </Button>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    @apply flex flex-col gap-4 w-full;
  }
</style>
