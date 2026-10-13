<script lang="ts">
import { MarkdownContent, taskProgress, toggleTask } from "#core/markdown";
import Button from "#shared/ui/Button.svelte";
import { HoverPopover } from "#shared/ui/hoverPopover.svelte.ts";
import IconBars3BottomLeft from "#shared/ui/icons/IconBars3BottomLeft.svelte";
import { createSetTodoDescriptionMutation } from "../api/mutations";
import type { Todo } from "../model/types";

/**
 * A todo's description, in little: an icon after its title, then the description on hover or click, to read it, check
 * its tasks, or edit it.
 */
type TodoDescriptionMarkerProps = {
  /** With a description. */
  todo: Todo;
  /** Opens its edit modal, to edit the description. */
  onEdit: () => void;
};
const { todo, onEdit }: TodoDescriptionMarkerProps = $props();

const panelId = $props.id();
const description = $derived(todo.description ?? "");
const progress = $derived(taskProgress(description));

const popover = new HoverPopover();
let body = $state<HTMLDivElement>();

// Shown checked right away: tells if saving fails, then shows the saved description back
const setDescription = createSetTodoDescriptionMutation(() => todo, {
  optimistic: true,
});
const onToggleTask = (task: Parameters<typeof toggleTask>[1]) => {
  const edit = toggleTask(description, task);
  if (!edit) return false;
  setDescription.mutate(
    description.slice(0, edit.from) + edit.insert + description.slice(edit.to),
  );
  return true;
};
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
    aria-label={`Description of ${todo.title}`}
    aria-expanded={popover.open}
    aria-controls={panelId}
    onclick={popover.onclick}
  >
    <IconBars3BottomLeft />
  </Button>
  <div
    bind:this={popover.panel}
    id={panelId}
    popover="manual"
    role="dialog"
    aria-label={`Description of ${todo.title}`}
    class="panel"
  >
    <span class="heading">Description</span>
    <div class="body" bind:this={body}>
      <MarkdownContent content={description} {onToggleTask} scroller={body} />
    </div>
    <div class="footer">
      {#if progress.total > 0}
        <span class="tasks">{progress.done} of {progress.total} tasks done</span>
      {/if}
      <button
        type="button"
        class="edit"
        onclick={() => {
          popover.open = false;
          onEdit();
        }}
      >
        Edit description →
      </button>
    </div>
  </div>
</span>

<style lang="postcss">
  @reference "tailwindcss";

  .group {
    @apply flex shrink-0;
  }

  /* `Button`'s, muted as the links' marker, unlike the notes' accent: styled through the group, as it's another
     component's element */
  .group :global(.marker) {
    @apply items-center px-1 text-slate-400 transition-colors hover:text-white;

    &:global(.open) {
      @apply bg-white/10 text-white;
    }

    > :global(svg) {
      @apply size-3.5;
    }
  }

  /* As the note's marker's panel */
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

  /* A long description scrolls within, rather than growing past the window */
  .body {
    @apply max-h-60 overflow-y-auto;
  }

  .footer {
    @apply flex items-center justify-between gap-3 border-t border-white/10 pt-2;
  }

  .tasks {
    /* Dimmer than the rest, yet readable enough (WCAG AA) */
    @apply text-[11px]/4 text-slate-400 tabular-nums;
  }

  .edit {
    @apply ml-auto shrink-0 cursor-pointer rounded text-xs/4 font-semibold text-pink-300 transition-colors hover:text-pink-200;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
  }
</style>
