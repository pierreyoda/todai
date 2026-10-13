<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  import { createSetTodoDescriptionMutation } from "../api/mutations";
  import type { Todo } from "../model/types";
  import { Autosave } from "#shared/utils/autosave.svelte.ts";
  import { Confirmation } from "#shared/utils/confirmation.svelte.ts";
  import { beforeWorkspaceSwitch } from "#domains/workspaces";
  import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
  import SaveStatus from "#shared/ui/SaveStatus.svelte";
  import UnsavedLeaveModal from "#shared/ui/UnsavedLeaveModal.svelte";
  import { MarkdownCheatsheet } from "#core/markdown";
  import { MarkdownSplitEditor } from "#core/markdown";

  /**
   * A todo's description, in Markdown, saved on its own as it's written: e.g. in its edit modal, whose other fields are
   * submitted. Takes its container's height: the container must be a flex column of bounded height.
   */
  type TodoDescriptionFieldProps = {
    todo: Todo;
    /** Whether to focus its editor once shown. */
    autofocus?: boolean;
  };
  const { todo, autofocus = false }: TodoDescriptionFieldProps = $props();

  const setDescription = createSetTodoDescriptionMutation(() => todo);
  /** When it was last saved from here: the todo's `updatedAt` also changes with its other fields. */
  let savedAt = $state<Date>();
  // svelte-ignore state_referenced_locally: the draft starts from the todo's description, then is the one written
  const autosave = new Autosave(todo.description ?? "", async (description) => {
    await setDescription.mutateAsync(description);
    savedAt = new Date();
  });

  // Dismissed until the next failure
  let errorDismissed = $state(false);
  $effect(() => {
    if (autosave.status !== "error") errorDismissed = false;
  });

  /** Whether to leave although the draft couldn't be saved. */
  const leaving = new Confirmation();
  /** Once left without it for another workspace: the draft must not be saved in that one's database. */
  let draftDiscarded = false;

  /**
   * Saves the description before leaving it (e.g. closing its modal), or asks to leave without it if that fails:
   * whether to leave.
   */
  export const saveBeforeLeaving = async () =>
    (await autosave.flush()) || (await leaving.ask());

  // Switching to another workspace too
  onMount(() =>
    beforeWorkspaceSwitch(async () => {
      const leave = await saveBeforeLeaving();
      if (leave && autosave.dirty) draftDiscarded = true;
      return leave;
    }),
  );

  onMount(() => {
    // Closing the window too. Quitting the app (⌘Q) doesn't: it can't be intercepted.
    const unlisten = getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await saveBeforeLeaving())) event.preventDefault();
    });
    return () => {
      void unlisten.then((stop) => stop());
      // Destroyed without being left first (e.g. its todo filtered out of the list): saved in the background
      if (!draftDiscarded) void autosave.flush();
    };
  });
</script>

<div class="field">
  {#if autosave.status === "error" && !errorDismissed}
    <ErrorBanner
      title="Could not save the description"
      error={autosave.error}
      hint="Your text is kept here: it's saved again with your next edit."
      action={{ label: "Try again", onClick: () => void autosave.flush() }}
      onDismiss={() => (errorDismissed = true)}
    />
  {/if}
  <MarkdownSplitEditor
    bind:value={autosave.draft}
    label="Description"
    placeholder={"Details, steps, links…\n\n# Headings, **bold**, - lists, - [ ] tasks"}
    {autofocus}
    emptyTitle="No description yet"
    emptyHint="it shows up here, rendered, and saves as you type."
  >
    {#snippet heading(tabs)}
      <div class="heading">
        <!-- The editor's label, for screen readers, names it too -->
        <span class="label" aria-hidden="true">Description</span>
        {#if tabs}
          {@render tabs()}
        {:else}
          <span class="syntax">Markdown</span>
        {/if}
        <div class="status">
          <SaveStatus status={autosave.status} {savedAt} />
          <MarkdownCheatsheet />
        </div>
      </div>
    {/snippet}
  </MarkdownSplitEditor>
</div>
<UnsavedLeaveModal
  bind:show={leaving.shown}
  name="This description"
  leaving="close"
  error={autosave.error}
  onLeave={() => leaving.answer(true)}
/>

<style lang="postcss">
  @reference "tailwindcss";

  .field {
    @apply flex min-h-0 flex-1 flex-col gap-3;
  }

  .heading {
    @apply flex items-center gap-2;
  }

  /* As the form's labels */
  .label {
    @apply text-sm/5 font-medium text-slate-300;
  }

  .syntax {
    @apply text-xs/4 text-slate-500;
  }

  .status {
    @apply -my-1 ml-auto flex items-center gap-2;
  }
</style>
