<script lang="ts">
  import IconPencilSquare from "#shared/ui/icons/IconPencilSquare.svelte";
  import MarkdownContent from "./MarkdownContent.svelte";

  type MarkdownPreviewProps = {
    /** The Markdown, as being written. */
    content: string;
    /**
     * Checks or unchecks the task shown `index`-th, `checked` before, with `text` as its text. Returns whether it did:
     * not if its line can't be found in `content`.
     */
    onToggleTask: (task: { index: number; checked: boolean; text: string }) => boolean;
    /** Shown while `content` is blank, e.g. "A blank page for today". */
    emptyTitle: string;
    /** Under `emptyTitle`, e.g. where to write. */
    emptyHint: string;
    /** Keeps its "Preview" title for screen readers only, e.g. while tabs name it. */
    hideTitle?: boolean;
  };
  const { content, onToggleTask, emptyTitle, emptyHint, hideTitle = false }: MarkdownPreviewProps = $props();

  const titleId = $props.id();
  const blank = $derived(content.trim() === "");

  let surface = $state<HTMLElement>();

  /** Scrolls to `ratio` of its content, from 0 (its top shown) to 1 (its bottom shown). */
  export const scrollToRatio = (ratio: number) => {
    if (!surface) return;
    surface.scrollTop = ratio * (surface.scrollHeight - surface.clientHeight);
  };
</script>

<section class="preview" aria-labelledby={titleId}>
  <h2 id={titleId} class={["title", hideTitle && "sr-only"]}>Preview</h2>
  <div class="surface" bind:this={surface}>
    {#if blank}
      <div class="empty" role="status">
        <div class="disc" aria-hidden="true">
          <IconPencilSquare class="size-11" />
        </div>
        <h3 class="empty-title">{emptyTitle}</h3>
        <p class="hint">{emptyHint}</p>
      </div>
    {:else}
      <MarkdownContent {content} {onToggleTask} scroller={surface} />
    {/if}
  </div>
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .preview {
    @apply flex min-h-0 flex-col gap-1.5;
  }

  /* As the editor's label */
  .title {
    @apply text-sm font-medium text-slate-300;
  }

  /* Surface: barely lifted from the background, scrolling on its own. Positioned, so that the rendered elements
     positioned absolutely (e.g. screen readers' texts) scroll with it, rather than overflowing the page. */
  .surface {
    @apply relative min-h-0 flex-1 overflow-y-auto rounded-xl bg-white/3 px-5 py-4 ring-1 ring-white/10;
  }

  .empty {
    @apply flex h-full flex-col items-center justify-center gap-2 p-6 text-center;
  }

  /* As the empty todos' disc, smaller */
  .disc {
    @apply mb-4 flex size-24 items-center justify-center rounded-full text-pink-400;
    @apply bg-[radial-gradient(circle,color-mix(in_oklab,var(--color-pink-400)_22%,transparent),transparent_70%)];
    @apply ring-1 ring-white/10 ring-inset;
    animation: glow 4s ease-in-out infinite;
    @apply motion-reduce:animate-none;
  }

  .empty-title {
    @apply text-xl font-semibold text-white;
  }

  .hint {
    @apply max-w-72 text-sm text-slate-400;
  }

  @keyframes glow {
    0%,
    100% {
      box-shadow: 0 0 24px -8px var(--color-pink-400);
    }
    50% {
      box-shadow: 0 0 40px -6px var(--color-pink-400);
    }
  }
</style>
