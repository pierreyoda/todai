<script lang="ts">
  import type { Workspace } from "#domains/workspaces";
  import { createSetLinksBehaviorsMutation } from "../api/mutations";
  import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
  import FieldCheckbox from "#shared/ui/FieldCheckbox.svelte";

  type LinksBehaviorSettingsProps = {
    workspace: Workspace;
  };
  const { workspace }: LinksBehaviorSettingsProps = $props();

  const legendId = $props.id();
  const update = createSetLinksBehaviorsMutation();

  // Follow the workspace's settings (once saved, or for another workspace), edited here until then
  let autoRefresh = $derived(workspace.linksAutoRefresh);
  let suggestDone = $derived(workspace.linksSuggestDone);
  let pasteToCreate = $derived(workspace.linksPasteToCreate);

  const save = () =>
    update.mutate({
      id: workspace.id,
      linksAutoRefresh: autoRefresh,
      linksSuggestDone: suggestDone,
      linksPasteToCreate: pasteToCreate,
    });
</script>

<section class="settings" aria-labelledby={legendId}>
  <h3 id={legendId} class="title">Behavior</h3>
  {#if update.isError}
    <ErrorBanner
      title="Could not save the links settings"
      error={update.error}
      onDismiss={() => update.reset()}
    />
  {/if}
  <FieldCheckbox
    label="Keep links up to date while todai is open"
    description="Open pull requests and issues: every 10 minutes, and when you come back to todai. Merged or closed ones: once a day. Otherwise, only when a link is added, or refreshed from its details."
    bind:checked={autoRefresh}
    disabled={update.isPending}
    onchange={save}
  />
  <FieldCheckbox
    label="Suggest completing a todo once its link is done"
    description="A merged pull request or a completed issue shows “mark as done” on its todo. Nothing is ever completed on its own."
    bind:checked={suggestDone}
    disabled={update.isPending}
    onchange={save}
  />
  <FieldCheckbox
    label="Offer to create a todo from a pasted link"
    description="In “Add a new todo item”, a pasted pull request or issue lends its title to the todo."
    bind:checked={pasteToCreate}
    disabled={update.isPending}
    onchange={save}
  />
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .settings {
    /* Layout */
    @apply flex flex-col gap-4 p-4;
    /* Surface: barely lifted from the background, like the cards above */
    @apply rounded-xl bg-white/3 ring-1 ring-white/10;
  }

  .title {
    @apply text-sm/5 font-semibold text-white;
  }
</style>
