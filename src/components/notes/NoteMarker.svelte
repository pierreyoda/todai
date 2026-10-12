<script lang="ts">
  import type { Day, Note } from "../../client/types";
  import { formatDateTime, formatFullDay } from "../../utils/dates";
  import { computePosition } from "../common/dropdown/positioning";
  import Button from "../common/Button.svelte";
  import IconDocumentText from "../common/icons/IconDocumentText.svelte";
  import NoteContent from "./NoteContent.svelte";

  /** A day's note, in little: an icon after the day, then the note on hover or click, to read it or open it. */
  type NoteMarkerProps = {
    note: Note;
    /** Its note opens as Today's sub-page, the others' as the calendar's. */
    today: Day;
  };
  const { note, today }: NoteMarkerProps = $props();

  /** How long hovering takes to show the note, and leaving it to hide it. */
  const HOVER_OPEN_MS = 150;
  const HOVER_CLOSE_MS = 200;

  const panelId = $props.id();
  const dayName = $derived(formatFullDay(note.day));
  const href = $derived(
    note.day === today ? "/today/notes" : `/calendar/notes/${note.day}`,
  );

  let open = $state(false);
  let button = $state<HTMLButtonElement>();
  let panel = $state<HTMLDivElement>();
  let body = $state<HTMLDivElement>();

  let timer: ReturnType<typeof setTimeout> | undefined;
  const later = (action: () => void, delay: number) => {
    clearTimeout(timer);
    timer = setTimeout(action, delay);
  };
  $effect(() => () => clearTimeout(timer));

  const position = () => {
    if (!button || !panel) return;
    const { x, y } = computePosition({
      anchor: button.getBoundingClientRect(),
      floating: panel.getBoundingClientRect(),
      viewport: {
        width: document.documentElement.clientWidth,
        height: document.documentElement.clientHeight,
      },
      placement: "bottom-start",
      offset: 4,
    });
    panel.style.left = `${x}px`;
    panel.style.top = `${y}px`;
  };

  // A manual popover, as the links' marker: hovering it leaves any other one open (e.g. a menu)
  $effect(() => {
    if (!panel) return;
    const shown = panel.matches(":popover-open");
    if (open && !shown) {
      panel.showPopover();
      // Positioned in the same task as the opening, so it never paints at the default position
      position();
    } else if (!open && shown) {
      panel.hidePopover();
    }
  });

  // Closed by Escape or by clicking elsewhere, as it's not light-dismissed
  $effect(() => {
    if (!open) return;
    const onpointerdown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!button?.contains(target) && !panel?.contains(target)) open = false;
    };
    const onkeydown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      const hadFocus = panel?.contains(document.activeElement);
      open = false;
      if (hadFocus) button?.focus();
    };
    document.addEventListener("pointerdown", onpointerdown, true);
    document.addEventListener("keydown", onkeydown);
    window.addEventListener("resize", position);
    window.addEventListener("scroll", position, { capture: true, passive: true });
    return () => {
      document.removeEventListener("pointerdown", onpointerdown, true);
      document.removeEventListener("keydown", onkeydown);
      window.removeEventListener("resize", position);
      window.removeEventListener("scroll", position, { capture: true });
    };
  });
</script>

<!-- Hovering the marker or the note keeps it shown: the note is part of the group, though shown on top -->
<span
  class="group"
  role="presentation"
  onpointerenter={(event) => {
    if (event.pointerType === "mouse") later(() => (open = true), HOVER_OPEN_MS);
  }}
  onpointerleave={(event) => {
    if (event.pointerType === "mouse") later(() => (open = false), HOVER_CLOSE_MS);
  }}
>
  <Button
    bind:ref={button}
    style="plain"
    size="xs"
    class={["marker", open && "open"]}
    aria-label={`Note of ${dayName}`}
    aria-expanded={open}
    aria-controls={panelId}
    onclick={(event) => {
      clearTimeout(timer);
      // Clicked once hovered: kept shown. With the keyboard: shown or hidden.
      open = event.detail > 0 ? true : !open;
    }}
  >
    <IconDocumentText />
  </Button>
  <div
    bind:this={panel}
    id={panelId}
    popover="manual"
    role="dialog"
    aria-label={`Note of ${dayName}`}
    class="panel"
  >
    <span class="heading">Note · {dayName}</span>
    <!-- Read only: tasks are checked from the note's page -->
    <div class="body" bind:this={body}>
      <NoteContent content={note.content} scroller={body} />
    </div>
    <div class="footer">
      <time class="edited" datetime={note.updatedAt.toISOString()}>
        Edited {formatDateTime(note.updatedAt)}
      </time>
      <a class="open" {href} onclick={() => (open = false)}>Open note →</a>
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
