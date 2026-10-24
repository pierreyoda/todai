<script lang="ts">
import type { Snippet } from "svelte";
import Button from "./Button.svelte";

type ModalProps = {
  show: boolean;
  /**
   * `md` (by default) fits a form or a message. `lg` takes most of the window, up to 64rem × 52rem, for its content
   * to fill: e.g. an editor. `xl` likewise, up to 72rem × 80rem. Its body is then a flex column of bounded height.
   */
  size?: "md" | "lg" | "xl";
  header?: Snippet;
  children: Snippet;
  /** Footer buttons, right-aligned (stacked on small screens). */
  actions?: Snippet;
  /**
   * Asked before closing it from its close button, its backdrop or Escape: it closes only once this resolves to `true`,
   * e.g. once what's edited in it is saved. Closing it otherwise, by setting `show`, isn't asked.
   */
  onCloseRequest?: () => boolean | Promise<boolean>;
};
let {
  show = $bindable(),
  size = "md",
  header,
  children,
  actions,
  onCloseRequest,
}: ModalProps = $props();

const titleId = $props.id();

let dialog = $state<HTMLDialogElement>();
$effect(() => {
  if (show) dialog?.showModal();
  else dialog?.close();
});

/** While `onCloseRequest` is answering: asked to close again meanwhile, it isn't asked twice. */
let requesting = false;

/** Closes it as asked from it (close button, backdrop, Escape), if `onCloseRequest` agrees. */
const requestClose = async () => {
  if (!onCloseRequest) {
    dialog?.close();
    return;
  }
  if (requesting) return;
  requesting = true;
  try {
    if (await onCloseRequest()) dialog?.close();
  } catch (error) {
    console.error("Could not tell whether to close the modal:", error);
  } finally {
    requesting = false;
  }
};

/** Escape is left to a popover shown in it (e.g. a menu), which it closes first. */
const hasPopoverOpen = () => !!dialog?.querySelector(":popover-open");

// Asked rather than closed by the browser: from the document, as Escape may also be pressed with nothing focused
// (e.g. once it left an editor), and handled before the browser, whose own close request can't always be cancelled
// (Chromium closes the dialog anyway when asked again without the user interacting with the page in between).
$effect(() => {
  if (!show || !onCloseRequest) return;
  const onkeydown = (event: KeyboardEvent) => {
    if (event.key !== "Escape" || event.defaultPrevented || event.isComposing)
      return;
    // Pressed in another modal, shown over it
    const target =
      event.target instanceof Element ? event.target.closest("dialog") : null;
    if ((target && target !== dialog) || hasPopoverOpen()) return;
    event.preventDefault();
    void requestClose();
  };
  document.addEventListener("keydown", onkeydown);
  return () => document.removeEventListener("keydown", onkeydown);
});
</script>

<dialog
  bind:this={dialog}
  data-size={size}
  aria-labelledby={header ? titleId : undefined}
  onclose={() => (show = false)}
  oncancel={(event) => {
    // Any other close request, e.g. a popover's Escape: asked too
    if (!onCloseRequest) return;
    event.preventDefault();
    if (!hasPopoverOpen()) void requestClose();
  }}
  onclick={(e) => {
    if (e.target === dialog) void requestClose();
  }}
>
  <div class="panel">
    <header>
      {#if header}
        <h2 id={titleId} class="title">{@render header()}</h2>
      {/if}
      <Button
        style="plain"
        class="close"
        aria-label="Close"
        onclick={() => void requestClose()}
      >
        <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path
            d="M6.28 5.22a.75.75 0 0 0-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 1 0 1.06 1.06L10 11.06l3.72 3.72a.75.75 0 1 0 1.06-1.06L11.06 10l3.72-3.72a.75.75 0 0 0-1.06-1.06L10 8.94 6.28 5.22Z"
          />
        </svg>
      </Button>
    </header>

    <div class="body">
      {@render children()}
    </div>

    {#if actions}
      <footer>
        {@render actions()}
      </footer>
    {/if}
  </div>
</dialog>

<style lang="postcss">
  @reference "tailwindcss";

  dialog {
    /* In the top layer: centered in the viewport (preflight resets margin to 0) */
    @apply fixed inset-0 m-auto;
    /* Size: as wide as allowed, up to its size's */
    @apply w-full max-w-lg;
    /* Surface */
    @apply rounded-2xl border border-white/10 bg-slate-800 p-0 shadow-2xl;
    /* Typography */
    @apply text-slate-300;
    /* Size */
    &[data-size="lg"] {
      @apply h-[min(52rem,calc(100dvh-2rem))] w-[min(64rem,calc(100vw-4rem))] max-w-none;
    }
    &[data-size="xl"] {
      @apply h-[min(80rem,calc(100dvh-2rem))] w-[min(72rem,calc(100vw-4rem))] max-w-none;
    }
    /* Literal values: ::backdrop may not inherit the theme variables */
    &::backdrop {
      background: rgb(2 6 23 / 0.6);
      backdrop-filter: blur(4px);
    }
    &[open] {
      animation: enter 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
    }
    &[open]::backdrop {
      animation: fade 0.2s ease-out;
    }
  }

  .panel {
    @apply flex flex-col gap-4 p-6;
  }

  /* Large and extra large: its body takes the height left, for its content to fill. Children only: a modal shown from
     its content is another one. */
  dialog:is([data-size="lg"], [data-size="xl"]) > .panel {
    @apply h-full min-h-0;
    > .body {
      @apply flex min-h-0 flex-1 flex-col;
    }
  }

  header {
    @apply flex items-start justify-between gap-4;
  }

  .title {
    @apply text-lg/7 font-semibold text-white;
  }

  header :global(.close) {
    @apply -m-1.5 ml-auto shrink-0 items-center! border-0! p-1.5! text-slate-400!;
    @apply hover:bg-white/5! hover:text-white! active:bg-white/10!;
    > svg {
      @apply size-5;
    }
  }

  .body {
    @apply text-sm/6;
  }

  footer {
    @apply mt-2 flex flex-col-reverse gap-3 sm:flex-row sm:justify-end;
  }

  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(0.5rem) scale(0.96);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    dialog[open],
    dialog[open]::backdrop {
      animation: none;
    }
  }
</style>
