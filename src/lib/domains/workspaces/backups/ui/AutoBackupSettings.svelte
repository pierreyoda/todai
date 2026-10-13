<script lang="ts">
  import type { Workspace } from "../../model/types";
  import { createSetAutoBackupMutation } from "../api/mutations";
  import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
  import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";
  import FieldText from "#shared/ui/FieldText.svelte";

  /** Most automatic backups kept, as enforced by the backend. */
  const KEEP_MAX = 100;

  type AutoBackupSettingsProps = {
    workspace: Workspace;
  };
  const { workspace }: AutoBackupSettingsProps = $props();

  const update = createSetAutoBackupMutation();

  // Follow the workspace's settings (once saved, or for another workspace), edited here until then
  let autoBackup = $derived(workspace.autoBackup);
  let keepDraft = $derived(String(workspace.autoBackupKeep));
  const keep = $derived(Number(keepDraft.trim()));
  const keepValid = $derived(
    Number.isInteger(keep) && keep >= 1 && keep <= KEEP_MAX,
  );

  const save = () => {
    if (!keepValid) return;
    if (
      autoBackup === workspace.autoBackup &&
      keep === workspace.autoBackupKeep
    ) {
      return;
    }
    update.mutate({ id: workspace.id, autoBackup, autoBackupKeep: keep });
  };
</script>

<section class="settings" aria-label="Automatic backups">
  {#if update.isError}
    <ErrorBanner
      title="Could not save the automatic backups settings"
      error={update.error}
      onDismiss={() => update.reset()}
    />
  {/if}
  <div class="fields">
    <FieldCheckbox
      label="Back up daily"
      description={`When ${workspace.name} is opened, and every hour while it's open: at most once a day.`}
      bind:checked={autoBackup}
      disabled={update.isPending}
      onchange={save}
    />
    <!-- Saved when committed (Enter, or leaving the field), rather than at each keystroke -->
    <div class="keep">
      Keep the last
      <span class="keep-field">
        <FieldText
          label="Number of automatic backups kept"
          hideLabel
          inputmode="numeric"
          autocomplete="off"
          bind:value={keepDraft}
          aria-invalid={!keepValid}
          title={`From 1 to ${KEEP_MAX}: older automatic backups are deleted`}
          disabled={update.isPending}
          onchange={save}
        />
      </span>
      automatic backups
    </div>
  </div>
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .settings {
    /* Layout */
    @apply flex shrink-0 flex-col gap-3 p-4;
    /* Surface: barely lifted from the background */
    @apply rounded-xl bg-white/3 ring-1 ring-white/10;
  }

  .fields {
    @apply flex flex-wrap items-center justify-between gap-x-6 gap-y-3;
  }

  .keep {
    @apply flex items-center gap-2 text-sm/6 text-slate-300;
  }

  /* Wide enough for 3 digits */
  .keep-field {
    @apply w-16 tabular-nums;
  }
</style>
