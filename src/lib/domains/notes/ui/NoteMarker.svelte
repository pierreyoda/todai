<script lang="ts">
import { MarkdownContent } from "#core/markdown";
import type { Day } from "#core/time";
import { formatFullDay } from "#core/time";
import Button from "#shared/ui/Button.svelte";
import { HoverPopover } from "#shared/ui/hoverPopover.svelte.ts";
import IconDocumentText from "#shared/ui/icons/IconDocumentText.svelte";
import { formatDateTime } from "#shared/utils/timestamps.ts";
import { notePath } from "../model/paths";
import type { Note } from "../model/types";

/** A day's note, in little: an icon after the day, then the note on hover or click, to read it or open it. */
type NoteMarkerProps = {
  note: Note;
  /** Its note opens as Today's sub-page, the others' as the calendar's. */
  today: Day;
};
const { note, today }: NoteMarkerProps = $props();

const panelId = $props.id();
const dayName = $derived(formatFullDay(note.day));
const href = $derived(notePath(note.day, today));

const popover = new HoverPopover();
let body = $state<HTMLDivElement>();
</script>

<span
  class="group"
  role="presentation"
  onpointerenter={popover.onpointerenter}
  onpointerleave={popover.onpointerleave}
>
  <Button
    bind:ref={popover.button}
    style="plain"
    size="xs"
    class={["marker", popover.open && "open"]}
    aria-label={`Note of ${dayName}`}
    aria-expanded={popover.open}
    aria-controls={panelId}
    onclick={popover.onclick}
  >
    <IconDocumentText />
  </Button>
  <div
    bind:this={popover.panel}
    id={panelId}
    popover="manual"
    role="dialog"
    aria-label={`Note of ${dayName}`}
    class="panel"
  >
    <span class="heading">Note · {dayName}</span>
    <!-- Read only: tasks are checked from the note's page -->
    <div class="body" bind:this={body}>
      <MarkdownContent content={note.content} scroller={body} />
    </div>
    <div class="footer">
      <time class="edited" datetime={note.updatedAt.toISOString()}>
        Edited {formatDateTime(note.updatedAt)}
      </time>
      <a class="open" {href} onclick={() => (popover.open = false)}>Open note →</a>
    </div>
  </div>
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .group {
    @apply flex shrink-0;
  }

  /* `Button`'s, in the notes' accent, as the side panel's dot for today's: styled through the group, as it's another
     component's element */
  .group :global(.marker) {
    @apply items-center px-1 text-pink-400 transition-colors hover:text-pink-300;

    &:global(.open) {
      @apply bg-white/10 text-pink-300;
    }

    > :global(svg) {
      @apply size-3.5;
    }
  }

  .panel {
    /* Reset popover defaults: positioned by script */
    @apply inset-auto m-0 border-0;
    @apply w-80 max-w-[calc(100vw-2rem)] flex-col gap-2 p-3;
    @apply rounded-lg bg-slate-800/95 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;

    /* Only while shown: a display set otherwise would override the browser's hiding of closed popovers */
    &:popover-open {
      @apply flex;
    }
  }

  .heading {
    @apply text-[11px]/4 font-semibold tracking-wide text-slate-400 uppercase;
  }

  /* A long note scrolls within, rather than the note growing past the window */
  .body {
    @apply max-h-60 overflow-y-auto;
  }

  .footer {
    @apply flex items-center justify-between gap-3 border-t border-white/10 pt-2;
  }

  .edited {
    /* Dimmer than the rest, yet readable enough (WCAG AA) */
    @apply text-[11px]/4 text-slate-400;
  }

  .open {
    @apply shrink-0 rounded text-xs/4 font-semibold text-pink-300 transition-colors hover:text-pink-200;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
  }
</style>
