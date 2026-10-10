<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";

  import { createSaveNoteMutation } from "../../client/mutations";
  import type { Day, Note } from "../../client/types";
  import { Autosave } from "../../utils/autosave.svelte";
  import { formatFullDay } from "../../utils/dates";
  import { toggleTask } from "../../utils/markdownEditing";
  import NoteEditor from "./NoteEditor.svelte";
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

  onMount(() => {
    // Closing the window waits for the draft to be saved. Quitting the app (⌘Q) doesn't: it can't be intercepted.
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await autosave.flush();
    });
    return () => {
      void unlisten.then((stop) => stop());
      // Leaving the page: saved in the background, as the save outlives the page
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
    </div>
  </header>
  <div class="panes">
    <NoteEditor
      bind:this={editor}
      bind:value={autosave.draft}
      onScroll={syncPreviewScroll}
    />
    <NotePreview bind:this={preview} content={autosave.draft} {onToggleTask} />
  </div>
</section>

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
