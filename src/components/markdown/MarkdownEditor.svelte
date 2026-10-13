<script lang="ts">
  import {
    continueList,
    indent,
    outdent,
    toggleWrap,
    type TextEdit,
    type TextSelection,
  } from "../../utils/markdownEditing";
  import { tick } from "svelte";

  import FieldTextArea from "../common/FieldTextArea.svelte";

  type MarkdownEditorProps = {
    /** Bindable: the edited Markdown. */
    value: string;
    /** What to write about, then a reminder of the syntax. */
    placeholder: string;
    /** "Markdown" by default: what's written, e.g. "Description". */
    label?: string;
    /** Whether to focus it once shown, at the end of its text, to resume writing. */
    autofocus?: boolean;
    /** Keeps its label for screen readers only, e.g. while tabs or a heading name it. */
    hideLabel?: boolean;
    /** When scrolled, e.g. to scroll the preview along: see `scrollRatio`. */
    onScroll?: () => void;
  };
  let {
    value = $bindable(),
    placeholder,
    label = "Markdown",
    autofocus = false,
    hideLabel = false,
    onScroll,
  }: MarkdownEditorProps = $props();

  let textarea = $state<HTMLTextAreaElement>();

  const isMac = navigator.platform.startsWith("Mac");

  $effect(() => {
    if (!textarea || !autofocus) return;
    const element = textarea;
    // Once its container is shown too: e.g. a modal, opened once its content is mounted
    void tick().then(() => {
      element.focus();
      element.setSelectionRange(element.value.length, element.value.length);
    });
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

  /** Where `offset` is once `edit` is made: moved with the text after it, or to the end of the text replacing it. */
  const offsetAfter = (offset: number, { from, to, insert }: TextEdit) =>
    offset <= from
      ? offset
      : offset >= to
        ? offset + insert.length - (to - from)
        : from + insert.length;

  /**
   * Makes `edit` from outside of the editor (e.g. a task checked in the preview), undoable like the others, leaving
   * the selection, the scroll position and the focus as they were.
   */
  export const applyExternalEdit = (edit: TextEdit) => {
    if (!textarea) return;
    const { selectionStart, selectionEnd, scrollTop } = textarea;
    const focused = document.activeElement;
    // Editing commands only apply to the focused element
    textarea.focus({ preventScroll: true });
    apply(textarea, {
      ...edit,
      selection: {
        start: offsetAfter(selectionStart, edit),
        end: offsetAfter(selectionEnd, edit),
      },
    });
    textarea.scrollTop = scrollTop;
    if (focused === textarea) return;
    if (
      focused instanceof HTMLElement &&
      focused !== document.body &&
      focused.isConnected
    ) {
      focused.focus({ preventScroll: true });
    } else {
      textarea.blur();
    }
  };

  /**
   * How far the text is scrolled, from 0 (its top shown) to 1 (its bottom shown). While it all fits, or while hidden,
   * where the caret is in it instead: as it's written.
   */
  export const scrollRatio = (): number => {
    if (!textarea) return 0;
    const scrollable = textarea.scrollHeight - textarea.clientHeight;
    if (scrollable > 0) return textarea.scrollTop / scrollable;
    return textarea.value.length > 0
      ? textarea.selectionEnd / textarea.value.length
      : 0;
  };
</script>

<FieldTextArea
  bind:value
  bind:ref={textarea}
  {label}
  {hideLabel}
  mono
  fill
  {placeholder}
  spellcheck
  {onkeydown}
  onscroll={onScroll}
/>
