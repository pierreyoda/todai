<script lang="ts">
  import IconExclamationTriangle from "./icons/IconExclamationTriangle.svelte";
  import IconXMark from "./icons/IconXMark.svelte";

  type ErrorBannerProps = {
    /** What failed, e.g. "Could not import the workspace". */
    title: string;
    /** As thrown: the backend rejects commands with its error message, as a string. */
    error: unknown;
    /** Shown after the error's message, e.g. what happens next. */
    hint?: string;
    /** A button to recover, e.g. "Try again". */
    action?: { label: string; onClick: () => void; disabled?: boolean };
    onDismiss?: () => void;
  };
  const { title, error, hint, action, onDismiss }: ErrorBannerProps = $props();

  const message = $derived(
    typeof error === "string"
      ? error
      : error instanceof Error
        ? error.message
        : undefined,
  );
</script>

<div class="banner" role="alert">
  <span class="icon">
    <IconExclamationTriangle class="size-5" />
  </span>
  <div class="text">
    <p class="title">{title}</p>
    {#if message}
      <p class="message">{message}</p>
    {/if}
    {#if hint}
      <p class="message">{hint}</p>
    {/if}
  </div>
  {#if action}
    <button
      type="button"
      class="action"
      disabled={action.disabled}
      onclick={action.onClick}
    >
      {action.label}
    </button>
  {/if}
  {#if onDismiss}
    <button
      type="button"
      class="dismiss"
      aria-label="Dismiss"
      onclick={onDismiss}
    >
      <IconXMark class="size-4" />
    </button>
  {/if}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .banner {
    /* Layout */
    @apply flex w-full items-start gap-3 p-3;
    /* Surface: tinted, with a soft red ring */
    @apply rounded-xl bg-red-500/10 ring-1 ring-red-500/30;
    /* Slides in, with a short shake to catch the eye */
    animation:
      enter 0.25s cubic-bezier(0.34, 1.56, 0.64, 1),
      shake 0.4s 0.25s ease-in-out;
    @apply motion-reduce:animate-none;
  }

  .icon {
    @apply shrink-0 rounded-full bg-red-500/15 p-1.5 text-red-400;
  }

  .text {
    @apply flex min-w-0 grow flex-col gap-0.5 pt-1;
  }

  .title {
    @apply text-sm font-medium text-red-300;
  }

  /* Messages hold paths: wrapped anywhere rather than overflowing */
  .message {
    @apply text-xs/5 wrap-anywhere text-red-200/70;
  }

  /* Tinted like the banner, centered on its first lines */
  .action {
    @apply mt-1 shrink-0 cursor-pointer rounded-md bg-red-500/15 px-2.5 py-1 text-xs font-semibold text-red-200;
    @apply transition-colors duration-150 hover:bg-red-500/25 hover:text-red-100 motion-reduce:transition-none;
    @apply focus-visible:outline-2 focus-visible:outline-red-400;
    @apply disabled:cursor-wait disabled:opacity-60;
  }

  .dismiss {
    @apply shrink-0 cursor-pointer rounded-md p-1 text-red-300/70;
    @apply transition-colors duration-150 hover:bg-red-500/15 hover:text-red-200 motion-reduce:transition-none;
    @apply focus-visible:outline-2 focus-visible:outline-red-400;
  }

  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(-0.25rem) scale(0.98);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  @keyframes shake {
    0%,
    100% {
      transform: translateX(0);
    }
    25% {
      transform: translateX(-3px);
    }
    75% {
      transform: translateX(3px);
    }
  }
</style>
