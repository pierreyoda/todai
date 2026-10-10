<script lang="ts">
  import IconPencilSquare from "../common/icons/IconPencilSquare.svelte";

  type NotePreviewProps = {
    /** The note's Markdown, as being written. */
    content: string;
  };
  const { content }: NotePreviewProps = $props();

  const titleId = $props.id();
  const blank = $derived(content.trim() === "");
</script>

<section class="preview" aria-labelledby={titleId}>
  <h2 id={titleId} class="title">Preview</h2>
  <div class="surface">
    {#if blank}
      <div class="empty" role="status">
        <div class="disc" aria-hidden="true">
          <IconPencilSquare class="size-11" />
        </div>
        <h3 class="empty-title">A blank page for today</h3>
        <p class="hint">Write on the left: it shows up here, rendered, and saves as you type.</p>
      </div>
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

  /* Surface: barely lifted from the background, scrolling on its own */
  .surface {
    @apply min-h-0 flex-1 overflow-y-auto rounded-xl bg-white/3 px-5 py-4 ring-1 ring-white/10;
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
