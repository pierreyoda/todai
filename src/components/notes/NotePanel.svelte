<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";

  import { beforeNavigate, goto } from "$app/navigation";

  import { createSaveNoteMutation } from "../../client/mutations";
  import type { Day, Note } from "../../client/types";
  import { Autosave } from "../../utils/autosave.svelte";
  import { formatFullDay } from "../../utils/dates";
  import { toggleTask } from "../../utils/markdownEditing";
  import ErrorBanner from "../common/ErrorBanner.svelte";
  import NoteCheatsheet from "./NoteCheatsheet.svelte";
  import NoteEditor from "./NoteEditor.svelte";
  import NoteLeaveModal from "./NoteLeaveModal.svelte";
  import NotePreview from "./NotePreview.svelte";
  import NoteSaveStatus from "./NoteSaveStatus.svelte";

  type NotePanelProps = {
    day: Day;
    /** The note of `day` (`null` if it has none), loaded before, then kept up to date with its saves. */
    note: Note | null;
  };
  const { day, note }: NotePanelProps = $props();

  const saveNote = createSaveNoteMutation();
  // svelte-ignore state_referenced_locally: the draft starts from the loaded note, then is the one written
  const autosave = new Autosave(note?.content ?? "", (content) =>
    saveNote.mutateAsync({ day, content }),
  );

  let editor = $state<ReturnType<typeof NoteEditor>>();
  let preview = $state<ReturnType<typeof NotePreview>>();

  // Through the editor, so that ⌘Z undoes it there
  const onToggleTask = (task: Parameters<typeof toggleTask>[1]) => {
    const edit = toggleTask(autosave.draft, task);
    if (!edit || !editor) return false;
    editor.applyExternalEdit(edit);
    return true;
  };

  /** The preview follows the editor: proportionally, as their contents' heights differ. */
  const syncPreviewScroll = () => {
    if (editor) preview?.scrollToRatio(editor.scrollRatio());
  };
  // Also once the preview shows a change, as it may have grown
  $effect(() => {
    void autosave.draft;
    void tick().then(syncPreviewScroll);
  });

  // Dismissed until the next failure
  let errorDismissed = $state(false);
  $effect(() => {
    if (autosave.status !== "error") errorDismissed = false;
  });

  let showLeaveModal = $state(false);
  /** Answers the leave modal shown, if any: `true` to leave anyway. */
  let answerLeaving: ((leave: boolean) => void) | null = null;
  /** Asks whether to leave although the draft couldn't be saved. */
  const confirmLeaving = () =>
    new Promise<boolean>((resolve) => {
      answerLeaving = resolve;
      showLeaveModal = true;
    });
  const answer = (leave: boolean) => {
    answerLeaving?.(leave);
    answerLeaving = null;
    showLeaveModal = false;
  };
  // Closed otherwise than by leaving (Stay, Escape, its backdrop): staying
  $effect(() => {
    if (!showLeaveModal) answer(false);
  });
  /** Once confirmed, leaving isn't held anymore. */
  let leavingAnyway = false;

  /** Saves the draft before leaving, or asks to leave without it if that fails: whether to leave. */
  const saveBeforeLeaving = async () => (await autosave.flush()) || (await confirmLeaving());

  // Leaving the page waits for the draft to be saved
  beforeNavigate((navigation) => {
    // Reloading or quitting can't wait: saved in the background, on destroy
    if (leavingAnyway || !autosave.dirty || navigation.type === "leave" || !navigation.to) return;
    navigation.cancel();
    const to = navigation.to.url;
    void saveBeforeLeaving().then((leave) => {
      if (!leave) return;
      leavingAnyway = true;
      void goto(to);
    });
  });

  onMount(() => {
    // Closing the window too. Quitting the app (⌘Q) doesn't: it can't be intercepted.
    const unlisten = getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await saveBeforeLeaving())) event.preventDefault();
    });
    return () => {
      void unlisten.then((stop) => stop());
      // Destroyed without navigating away first (e.g. reloading): saved in the background, as the save outlives it
      void autosave.flush();
    };
  });
</script>

<section class="container">
  <header class="header">
    <h1 class="title">Today’s note</h1>
    <time class="day" datetime={day}>{formatFullDay(day)}</time>
    <div class="actions">
      <NoteSaveStatus status={autosave.status} savedAt={note?.updatedAt} />
      <NoteCheatsheet />
    </div>
  </header>
  {#if autosave.status === "error" && !errorDismissed}
    <div class="px-2">
      <ErrorBanner
        title="Could not save today’s note"
        error={autosave.error}
        hint="Your text is kept here: it’s saved again with your next edit."
        action={{ label: "Try again", onClick: () => void autosave.flush() }}
        onDismiss={() => (errorDismissed = true)}
      />
    </div>
  {/if}
  <div class="panes">
    <NoteEditor
      bind:this={editor}
      bind:value={autosave.draft}
      onScroll={syncPreviewScroll}
    />
    <NotePreview bind:this={preview} content={autosave.draft} {onToggleTask} />
  </div>
</section>
<NoteLeaveModal bind:show={showLeaveModal} error={autosave.error} onLeave={() => answer(true)} />

<style lang="postcss">
  @reference "tailwindcss";

  .container {
    /* Takes the width left by the side panel */
    @apply flex h-full min-w-0 flex-1 flex-col gap-3 p-4;
  }

  .header {
    @apply flex items-center gap-3 px-2 pt-2;
  }

  .title {
    @apply text-lg/7 font-semibold text-white;
  }

  .day {
    @apply text-sm text-slate-400;
  }

  .actions {
    @apply ml-auto flex items-center gap-2;
  }

  /* Side by side, as tall as the page allows rather than their content: each scrolls on its own */
  .panes {
    @apply grid min-h-0 flex-1 grid-cols-2 grid-rows-1 gap-4 px-2 pb-2;
  }
</style>
