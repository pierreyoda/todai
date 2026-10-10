<script lang="ts">
  import {
    continueList,
    indent,
    outdent,
    toggleWrap,
    type TextEdit,
    type TextSelection,
  } from "../../utils/markdownEditing";
  import FieldTextArea from "../common/FieldTextArea.svelte";

  type NoteEditorProps = {
    /** Bindable: the note's Markdown. */
    value: string;
  };
  let { value = $bindable() }: NoteEditorProps = $props();

  let textarea = $state<HTMLTextAreaElement>();

  const isMac = navigator.platform.startsWith("Mac");

  // Focused once opened, at the end of the note, to resume writing
  $effect(() => {
    if (!textarea) return;
    textarea.focus();
    textarea.setSelectionRange(textarea.value.length, textarea.value.length);
  });

  /** Makes `edit` through the browser's editing commands, so that undo (⌘Z) still works after it. */
  const apply = (
    textarea: HTMLTextAreaElement,
    { from, to, insert, selection }: TextEdit,
  ) => {
    textarea.setSelectionRange(from, to);
    // `execCommand` is deprecated, but still supported everywhere, and nothing replaces it: it's the only way for a
    // script to edit a textarea within its native undo history. `setRangeText`, below, would leave the edit out of it.
    const applied =
      insert === ""
        ? from === to || document.execCommand("delete")
        : document.execCommand("insertText", false, insert);
    if (!applied) {
      textarea.setRangeText(insert, from, to);
      textarea.dispatchEvent(new Event("input", { bubbles: true }));
    }
    textarea.setSelectionRange(selection.start, selection.end);
  };

  /**
   * What a key does in the editor, if anything special: an edit (`null` if there's nothing to change), or `"blur"`.
   * `undefined` leaves the key to the browser.
   */
  const actionFor = (
    event: KeyboardEvent,
    text: string,
    selection: TextSelection,
  ): TextEdit | null | "blur" | undefined => {
    const modifier = isMac ? event.metaKey : event.ctrlKey;
    const otherModifier = isMac ? event.ctrlKey : event.metaKey;
    if (event.altKey || otherModifier) return undefined;
    if (modifier) {
      if (event.shiftKey) return undefined;
      switch (event.key.toLowerCase()) {
        case "b":
          return toggleWrap(text, selection, "**");
        case "i":
          return toggleWrap(text, selection, "_");
        default:
          return undefined;
      }
    }
    switch (event.key) {
      case "Enter":
        // A plain new line otherwise
        return event.shiftKey
          ? undefined
          : (continueList(text, selection) ?? undefined);
      case "Tab":
        // Kept in the editor even with nothing to indent: Escape lets the keyboard leave it
        return (event.shiftKey ? outdent : indent)(text, selection);
      case "Escape":
        return "blur";
      default:
        return undefined;
    }
  };

  const onkeydown = (event: KeyboardEvent) => {
    // Composing text with an input method (e.g. accents, CJK): its keys are its own
    if (event.isComposing || !textarea) return;
    const selection = {
      start: textarea.selectionStart,
      end: textarea.selectionEnd,
    };
    const action = actionFor(event, textarea.value, selection);
    if (action === undefined) return;
    event.preventDefault();
    if (action === "blur") textarea.blur();
    else if (action) apply(textarea, action);
  };
</script>

<FieldTextArea
  bind:value
  bind:ref={textarea}
  label="Markdown"
  mono
  fill
  placeholder={"Write about your day…\n\n# Headings, **bold**, - lists, - [ ] tasks"}
  spellcheck
  {onkeydown}
/>
