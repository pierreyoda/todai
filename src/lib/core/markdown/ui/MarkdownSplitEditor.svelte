<script lang="ts">
import { type Snippet, tick } from "svelte";
import type { ClassValue } from "svelte/elements";

import { toggleTask } from "../model/editing";
import MarkdownEditor from "./MarkdownEditor.svelte";
import MarkdownPreview from "./MarkdownPreview.svelte";

/**
 * Markdown written in an editor and shown rendered next to it, where its tasks can be checked. Too narrow for both
 * side by side, they're shown one at a time, through tabs.
 *
 * Takes its container's height: the container must be a flex column of bounded height.
 */
type MarkdownSplitEditorProps = {
  /** Bindable: the edited Markdown. */
  value: string;
  /** The editor's: what to write about, then a reminder of the syntax. */
  placeholder: string;
  /** The editor's, "Markdown" by default: what's written, e.g. "Description". */
  label?: string;
  /**
   * Above both panes, in place of their labels: e.g. the field's name, then what's about it. Given the tabs while
   * they're shown, to put among it.
   */
  heading?: Snippet<[tabs: Snippet | undefined]>;
  /** Whether to focus the editor once shown. */
  autofocus?: boolean;
  /** What the preview says while there's nothing to show, e.g. "A blank page for today". */
  emptyTitle: string;
  /**
   * What writing does, said after where to write, which depends on the layout: e.g. "it shows up here, rendered, and
   * saves as you type".
   */
  emptyHint: string;
  class?: ClassValue;
};
let {
  value = $bindable(),
  placeholder,
  label,
  heading,
  autofocus = false,
  emptyTitle,
  emptyHint,
  class: extraClass,
}: MarkdownSplitEditorProps = $props();

/** Below this width (36rem), each pane would be too narrow to write in. */
const NARROW_WIDTH = 576;

const id = $props.id();
/** Measured once shown: side by side until then. */
let width = $state(0);
const narrow = $derived(width > 0 && width < NARROW_WIDTH);

type Pane = "editor" | "preview";
const TABS: { pane: Pane; label: string }[] = [
  { pane: "editor", label: "Markdown" },
  { pane: "preview", label: "Preview" },
];
/** The pane shown while narrow. */
let shown = $state<Pane>("editor");

let editor = $state<ReturnType<typeof MarkdownEditor>>();
let preview = $state<ReturnType<typeof MarkdownPreview>>();

// Through the editor, so that ⌘Z undoes it there
const onToggleTask = (task: Parameters<typeof toggleTask>[1]) => {
  const edit = toggleTask(value, task);
  if (!edit || !editor) return false;
  editor.applyExternalEdit(edit);
  return true;
};

/** The preview follows the editor, while both are shown: proportionally, as their contents' heights differ. */
const syncPreviewScroll = () => {
  if (narrow) return;
  if (editor) preview?.scrollToRatio(editor.scrollRatio());
};
// Also once the preview shows a change, as it may have grown
$effect(() => {
  void value;
  void tick().then(syncPreviewScroll);
});

const show = (pane: Pane) => {
  if (pane === shown) return;
  // Where the editor is, read while it's still shown
  const ratio = editor?.scrollRatio() ?? 0;
  shown = pane;
  if (pane === "preview") void tick().then(() => preview?.scrollToRatio(ratio));
};

/** As tabs do: the arrows move to the previous or next one, Home and End to the first and last. */
const onTabKeydown = (event: KeyboardEvent) => {
  const index = TABS.findIndex(({ pane }) => pane === shown);
  const targets: Record<string, number> = {
    ArrowLeft: index - 1,
    ArrowRight: index + 1,
    Home: 0,
    End: TABS.length - 1,
  };
  const target = targets[event.key];
  if (target === undefined) return;
  event.preventDefault();
  const { pane } = TABS[(target + TABS.length) % TABS.length];
  show(pane);
  document.getElementById(`${id}-${pane}-tab`)?.focus();
};
</script>

{#snippet tabs()}
  <div class="tabs" role="tablist" aria-label="Markdown or its preview">
    {#each TABS as { pane, label } (pane)}
      <button
        type="button"
        role="tab"
        id="{id}-{pane}-tab"
        class="tab"
        aria-selected={shown === pane}
        aria-controls="{id}-{pane}"
        tabindex={shown === pane ? 0 : -1}
        onclick={() => show(pane)}
        onkeydown={onTabKeydown}
      >
        {label}
      </button>
    {/each}
  </div>
{/snippet}

<div class={["split", narrow && "narrow", heading && "headed", extraClass]} bind:clientWidth={width}>
  {#if heading}
    <div class="heading">{@render heading(narrow ? tabs : undefined)}</div>
  {:else if narrow}
    {@render tabs()}
  {/if}
  <div class="pane" id="{id}-editor" hidden={narrow && shown !== "editor"}>
    <MarkdownEditor
      bind:this={editor}
      bind:value
      {placeholder}
      {label}
      {autofocus}
      hideLabel={narrow || !!heading}
      onScroll={syncPreviewScroll}
    />
  </div>
  <div class="pane" id="{id}-preview" hidden={narrow && shown !== "preview"}>
    <MarkdownPreview
      bind:this={preview}
      content={value}
      {onToggleTask}
      {emptyTitle}
      emptyHint={`${narrow ? "Write in the Markdown tab" : "Write on the left"}: ${emptyHint}`}
      hideTitle={narrow || !!heading}
    />
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  /* Side by side, as tall as their container allows rather than their content: each scrolls on its own */
  .split {
    @apply grid min-h-0 flex-1 grid-cols-2 grid-rows-1 gap-4;

    /* Under a heading, closer to it than to each other */
    &.headed {
      @apply grid-rows-[auto_minmax(0,1fr)] gap-x-4 gap-y-1.5;
    }

    /* One at a time, under the tabs */
    &.narrow {
      @apply grid-cols-1 grid-rows-[auto_minmax(0,1fr)] gap-1.5;
    }
  }

  .heading {
    @apply col-span-full;
  }

  /* Its editor or preview takes its whole height */
  .pane {
    @apply grid min-h-0 grid-rows-1;
  }

  /* As a segmented control, in place of the panes' labels: the pane shown is lifted */
  .tabs {
    @apply inline-flex gap-0.5 justify-self-start rounded-lg bg-white/5 p-0.5 ring-1 ring-white/10 ring-inset;
  }

  .tab {
    @apply cursor-pointer rounded-md px-2.5 py-0.5 text-xs/5 font-semibold text-slate-400 transition-colors;
    @apply hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;

    &[aria-selected="true"] {
      @apply bg-white/10 text-white shadow-[inset_0_1px_--theme(--color-white/5%)];
    }
  }
</style>
