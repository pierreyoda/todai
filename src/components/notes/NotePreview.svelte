<script lang="ts">
  import SvelteMarkdown, { type MarkedExtension } from "@humanspeak/svelte-markdown";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { tick } from "svelte";

  import IconArrowTopRightOnSquare from "../common/icons/IconArrowTopRightOnSquare.svelte";
  import IconPencilSquare from "../common/icons/IconPencilSquare.svelte";
  import IconPhoto from "../common/icons/IconPhoto.svelte";

  type NotePreviewProps = {
    /** The note's Markdown, as being written. */
    content: string;
    /**
     * Checks or unchecks the task shown `index`-th, `checked` before, with `text` as its text. Returns whether it did:
     * not if its line can't be found in `content`.
     */
    onToggleTask: (task: { index: number; checked: boolean; text: string }) => boolean;
  };
  const { content, onToggleTask }: NotePreviewProps = $props();

  /** HTML isn't interpreted but shown as typed: a note can neither load remote content, nor navigate the app. */
  const NO_HTML: MarkedExtension = {
    tokenizer: { html: () => undefined, tag: () => undefined },
  };
  // The same array at each render: a new one would have the whole note parsed again
  const extensions = [NO_HTML];

  const titleId = $props.id();
  const blank = $derived(content.trim() === "");

  let surface = $state<HTMLElement>();

  /** Scrolls to `ratio` of its content, from 0 (its top shown) to 1 (its bottom shown). */
  export const scrollToRatio = (ratio: number) => {
    if (!surface) return;
    surface.scrollTop = ratio * (surface.scrollHeight - surface.clientHeight);
  };

  const taskBoxes = () => [...(surface?.querySelectorAll<HTMLInputElement>("input[data-task]") ?? [])];

  const onTaskClick = (event: MouseEvent, checked: boolean, text: string) => {
    const box = event.currentTarget as HTMLInputElement;
    const index = taskBoxes().indexOf(box);
    const focused = document.activeElement === box;
    if (!onToggleTask({ index, checked, text })) {
      // Left as it is
      event.preventDefault();
      return;
    }
    // Rendered again from the edited note: focus follows, for the keyboard
    if (focused) void tick().then(() => taskBoxes()[index]?.focus());
  };

  const isExternal = (href: string) => /^(https?|mailto|tel):/i.test(href);

  /** Opens external links in their default application, and in-note ones (`#heading`) here: never in the app. */
  const onLinkClick = (event: MouseEvent, href: string | undefined) => {
    event.preventDefault();
    if (!href) return;
    if (href.startsWith("#")) {
      const target = surface?.querySelector(`#${CSS.escape(decodeURIComponent(href.slice(1)))}`);
      // Rather than `scrollIntoView`, which would also scroll the page around it
      if (surface && target) {
        surface.scrollTop += target.getBoundingClientRect().top - surface.getBoundingClientRect().top;
      }
    } else if (isExternal(href)) {
      openUrl(href).catch((error) => console.error(`Could not open ${href}:`, error));
    }
  };
</script>

<section class="preview" aria-labelledby={titleId}>
  <h2 id={titleId} class="title">Preview</h2>
  <div class="surface" bind:this={surface}>
    {#if blank}
      <div class="empty" role="status">
        <div class="disc" aria-hidden="true">
          <IconPencilSquare class="size-11" />
        </div>
        <h3 class="empty-title">A blank page for today</h3>
        <p class="hint">Write on the left: it shows up here, rendered, and saves as you type.</p>
      </div>
    {:else}
      <div class="markdown">
        <SvelteMarkdown source={content} {extensions}>
          {#snippet listitem({ task, checked = false, text = "", children })}
            {#if task}
              <li class={["task", checked && "checked"]}>
                <span class="task-control">
                  <input
                    type="checkbox"
                    data-task
                    {checked}
                    aria-label={text.split("\n", 1)[0]}
                    onclick={(event) => onTaskClick(event, checked, text)}
                  />
                  <span class="task-box" aria-hidden="true">
                    <svg viewBox="0 0 14 14" fill="none">
                      <path d="M3 8L6 11L11 3.5" />
                    </svg>
                  </span>
                </span>
                <div class="task-content">{@render children?.()}</div>
              </li>
            {:else}
              <li>{@render children?.()}</li>
            {/if}
          {/snippet}
          {#snippet link({ href, title, children })}
            <!-- On one line: spaces would be underlined with the link -->
            <a {href} title={title ?? href} onclick={(event) => onLinkClick(event, href)}
              >{@render children?.()}{#if href && isExternal(href)}<IconArrowTopRightOnSquare
                  class="external"
                /><span class="sr-only"> (opens in your browser)</span>{/if}</a
            >
          {/snippet}
          {#snippet image({ href, title, text })}
            <!-- Not loaded: remote images would be fetched as the note is written -->
            <span class="image" title={title ?? href}>
              <IconPhoto class="shrink-0" />
              {text || "Image"}
            </span>
          {/snippet}
        </SvelteMarkdown>
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

  /* Surface: barely lifted from the background, scrolling on its own. Positioned, so that the note's elements
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

  /* The rendered note: its elements come from the Markdown renderers, out of this component's scope */
  .markdown {
    @apply text-sm/6 wrap-break-word text-slate-300;

    :global {
      /* Blocks, spaced evenly; headings set further apart from what precedes them */
      :where(p, ul, ol, blockquote, pre, table, hr, h1, h2, h3, h4, h5, h6) + * {
        @apply mt-3;
      }
      * + :where(h1, h2, h3, h4, h5, h6) {
        @apply mt-5;
      }

      h1 {
        @apply text-xl/7 font-semibold text-white;
      }
      h2 {
        @apply text-base/6 font-semibold text-white;
      }
      :where(h3, h4, h5, h6) {
        @apply text-sm/6 font-semibold text-white;
      }

      strong {
        @apply font-semibold text-white;
      }
      em {
        @apply italic;
      }
      del {
        @apply text-slate-500 line-through;
      }

      a {
        @apply cursor-pointer font-medium text-pink-400 underline decoration-pink-400/40 underline-offset-2;
        @apply transition-colors hover:text-pink-300 hover:decoration-pink-300;
      }
      .external {
        @apply ml-0.5 inline size-3 align-baseline;
      }

      /* As paths in the app's modals */
      code {
        @apply rounded bg-white/5 px-1 py-0.5 font-mono text-xs text-slate-200;
      }
      pre {
        @apply overflow-x-auto rounded-lg bg-slate-950/50 p-3 ring-1 ring-white/10;
        code {
          @apply bg-transparent p-0 text-xs/5;
        }
      }

      blockquote {
        @apply border-l-2 border-white/15 pl-3 text-slate-400 italic;
      }

      hr {
        @apply border-white/10;
      }

      ul {
        @apply list-disc pl-5;
      }
      ol {
        @apply list-decimal pl-5;
      }
      li::marker {
        @apply text-slate-500;
      }
      li + li,
      li > :where(ul, ol) {
        @apply mt-1;
      }

      /* As the app's tables */
      table {
        @apply w-full border-separate border-spacing-0 overflow-hidden rounded-lg text-left ring-1 ring-white/10;
      }
      th {
        @apply border-b border-white/10 px-3 py-2 text-xs font-semibold tracking-wider text-slate-400 uppercase;
      }
      td {
        @apply border-b border-white/5 px-3 py-2 tabular-nums;
      }
      tr:last-child td {
        @apply border-b-0;
      }
    }
  }

  .image {
    @apply inline-flex items-center gap-1.5 rounded-md bg-white/5 px-1.5 text-slate-400 ring-1 ring-white/10;
  }

  /* Task list items: their box where the bullet would be */
  .task {
    @apply -ml-5 flex list-none items-start gap-2.5;
  }

  .task-content {
    @apply min-w-0 flex-1;
  }

  /* Done: muted, and struck through unless it holds other items */
  .task.checked > .task-content {
    @apply text-slate-500;
    :global(:where(ul, ol)) {
      @apply text-slate-300;
    }
  }
  .task.checked:not(:has(:global(ul), :global(ol))) > .task-content {
    @apply line-through;
  }

  /* As `FieldCheckbox`, in the accent color: the native input sits invisibly on top of the box */
  .task-control {
    @apply relative mt-1 inline-flex shrink-0;
  }

  .task-control input {
    @apply absolute inset-0 z-10 m-0 size-full cursor-pointer appearance-none opacity-0;
  }

  .task-box {
    @apply relative flex size-4 items-center justify-center rounded-[0.3125rem] border border-white/15 bg-white/5;
    @apply transition-colors;
  }

  input:hover + .task-box {
    @apply border-white/30;
  }

  input:checked + .task-box {
    @apply border-white/5 bg-pink-500 shadow-[inset_0_1px_--theme(--color-white/15%)];
  }

  input:focus-visible + .task-box {
    @apply outline-2 outline-offset-2 outline-blue-500;
  }

  .task-box svg {
    @apply size-3.5 stroke-white stroke-2 opacity-0;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  input:checked + .task-box svg {
    @apply opacity-100;
  }
</style>
