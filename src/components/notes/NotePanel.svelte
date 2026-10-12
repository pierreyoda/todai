<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  import { beforeNavigate, goto } from "$app/navigation";

  import { createSaveNoteMutation } from "../../client/mutations";
  import type { Day, Note } from "../../client/types";
  import { Autosave } from "../../utils/autosave.svelte";
  import { dateToTodaiDate, formatFullDay } from "../../utils/dates";
  import { toggleTask } from "../../utils/markdownEditing";
  import { noteDrafts } from "../../utils/noteDrafts.svelte";
  import { beforeWorkspaceSwitch } from "../../utils/workspaceSwitch";
  import ErrorBanner from "../common/ErrorBanner.svelte";
  import SaveStatus from "../common/SaveStatus.svelte";
  import UnsavedLeaveModal from "../common/UnsavedLeaveModal.svelte";
  import MarkdownCheatsheet from "../markdown/MarkdownCheatsheet.svelte";
  import MarkdownSplitEditor from "../markdown/MarkdownSplitEditor.svelte";

  type NotePanelProps = {
    day: Day;
    /** The note of `day` (`null` if it has none), loaded before, then kept up to date with its saves. */
    note: Note | null;
  };
  const { day, note }: NotePanelProps = $props();

  // Today's, or a past day's, opened from the calendar: fixed once opened, like the day itself
  // svelte-ignore state_referenced_locally: fixed once mounted
  const isToday = day === dateToTodaiDate(new Date());
  // svelte-ignore state_referenced_locally: fixed once mounted
  const noteName = isToday ? "Today’s note" : `The note of ${formatFullDay(day)}`;

  const saveNote = createSaveNoteMutation();
  // svelte-ignore state_referenced_locally: the draft starts from the loaded note, then is the one written
  const autosave = new Autosave(note?.content ?? "", (content) =>
    saveNote.mutateAsync({ day, content }),
  );

  // Whether the day has a note then shows what's written, e.g. in the side panel, rather than waiting for its autosave
  onMount(() => noteDrafts.open(day, () => autosave.draft));

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

  /** Once left without it for another workspace: the draft must not be saved in that one's database. */
  let draftDiscarded = false;
  // Switching to another workspace too, which then shows its own note
  onMount(() =>
    beforeWorkspaceSwitch(async () => {
      const leave = await saveBeforeLeaving();
      if (leave && autosave.dirty) draftDiscarded = true;
      return leave;
    }),
  );

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
      if (!draftDiscarded) void autosave.flush();
    };
  });
</script>

<section class="container">
  <header class="header">
    {#if !isToday}
      <a href="/calendar" class="back">
        <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <path
            fill-rule="evenodd"
            d="M9.78 4.22a.75.75 0 0 1 0 1.06L7.06 8l2.72 2.72a.75.75 0 1 1-1.06 1.06L5.47 8.53a.75.75 0 0 1 0-1.06l3.25-3.25a.75.75 0 0 1 1.06 0Z"
            clip-rule="evenodd"
          />
        </svg>
        Calendar
      </a>
    {/if}
    <h1 class="title">{isToday ? "Today’s note" : "Note"}</h1>
    <time class="day" datetime={day}>{formatFullDay(day)}</time>
    <div class="actions">
      <SaveStatus status={autosave.status} savedAt={note?.updatedAt} />
      <MarkdownCheatsheet />
    </div>
  </header>
  {#if autosave.status === "error" && !errorDismissed}
    <div class="px-2">
      <ErrorBanner
        title={`Could not save ${isToday ? "today’s note" : "this note"}`}
        error={autosave.error}
        hint="Your text is kept here: it’s saved again with your next edit."
        action={{ label: "Try again", onClick: () => void autosave.flush() }}
        onDismiss={() => (errorDismissed = true)}
      />
    </div>
  {/if}
  <MarkdownSplitEditor
    bind:value={autosave.draft}
    class="px-2 pb-2"
    placeholder={"Write about your day…\n\n# Headings, **bold**, - lists, - [ ] tasks"}
    autofocus
    emptyTitle={`A blank page for ${isToday ? "today" : formatFullDay(day)}`}
    emptyHint="it shows up here, rendered, and saves as you type."
  />
</section>
<UnsavedLeaveModal
  bind:show={showLeaveModal}
  name={noteName}
  error={autosave.error}
  onLeave={() => answer(true)}
/>

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

  /* Back to the calendar, where a past day's note is opened from: as the side panel's muted links */
  .back {
    @apply -ml-1.5 inline-flex items-center gap-0.5 rounded-md py-0.5 pr-2 pl-1 text-sm text-slate-400 transition-colors;
    @apply hover:bg-white/5 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
    > svg {
      @apply size-4;
    }
  }

  .day {
    @apply text-sm text-slate-400;
  }

  .actions {
    @apply ml-auto flex items-center gap-2;
  }
</style>
