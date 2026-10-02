<script lang="ts">
  import type { Snippet } from "svelte";

  type ModalProps = {
    show: boolean;
    header?: Snippet;
    children: Snippet;
    /** Footer buttons, right-aligned (stacked on small screens). */
    actions?: Snippet;
  };
  let { show = $bindable(), header, children, actions }: ModalProps = $props();

  const titleId = $props.id();

  let dialog = $state<HTMLDialogElement>();
  $effect(() => {
    if (show) dialog?.showModal();
    else dialog?.close();
  });
</script>

<dialog
  bind:this={dialog}
  aria-labelledby={header ? titleId : undefined}
  onclose={() => (show = false)}
  onclick={(e) => {
    if (e.target === dialog) dialog.close();
  }}
>
  <div class="panel">
    <header>
      {#if header}
        <h2 id={titleId} class="title">{@render header()}</h2>
      {/if}
      <button
        type="button"
        class="close"
        aria-label="Close"
        onclick={() => dialog?.close()}
      >
        <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path
            d="M6.28 5.22a.75.75 0 0 0-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 1 0 1.06 1.06L10 11.06l3.72 3.72a.75.75 0 1 0 1.06-1.06L11.06 10l3.72-3.72a.75.75 0 0 0-1.06-1.06L10 8.94 6.28 5.22Z"
          />
        </svg>
      </button>
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
    /* Surface */
    @apply max-w-lg rounded-2xl border border-white/10 bg-slate-800 p-0 shadow-2xl;
    /* Typography */
    @apply text-slate-300;
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

  header {
    @apply flex items-start justify-between gap-4;
  }

  .title {
    @apply text-lg/7 font-semibold text-white;
  }

  /* Pushed right even without a title; negative margin keeps the icon aligned with the padding */
  .close {
    @apply -m-1.5 ml-auto shrink-0 rounded-lg p-1.5 text-slate-400;
    @apply hover:bg-white/5 hover:text-white;
    @apply focus:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
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
