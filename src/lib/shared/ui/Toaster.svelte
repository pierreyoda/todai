<script lang="ts">
import { cubicOut } from "svelte/easing";
import { fade, fly } from "svelte/transition";

import { toasts } from "../utils/toasts.svelte";
import Button from "./Button.svelte";
import IconExclamationTriangle from "./icons/IconExclamationTriangle.svelte";
import IconXMark from "./icons/IconXMark.svelte";
</script>

<!-- The app's `toasts`, at the bottom of the window. Paused while pointed at or focused, so that an action stays in
     reach. -->

<section
  class="toaster"
  aria-label="Notifications"
  onpointerenter={() => toasts.pause()}
  onpointerleave={() => toasts.resume()}
  onfocusin={() => toasts.pause()}
  onfocusout={(event) => {
    if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
      toasts.resume();
    }
  }}
>
  {#each toasts.items as toast (toast.id)}
    <div
      class={["toast", toast.tone]}
      role={toast.tone === "error" ? "alert" : "status"}
      in:fly={{ y: 16, duration: 200, easing: cubicOut }}
      out:fade={{ duration: 150 }}
    >
      {#if toast.tone === "error"}
        <IconExclamationTriangle class="shrink-0 text-red-400" />
      {/if}
      <p class="message">{toast.message}</p>
      {#if toast.action}
        <button type="button" class="action" onclick={() => toasts.act(toast.id)}>
          {toast.action.label}
        </button>
      {/if}
      <Button
        style="plain"
        size="xs"
        class="shrink-0 text-slate-400! hover:text-white!"
        aria-label="Dismiss"
        title="Dismiss"
        onclick={() => toasts.dismiss(toast.id)}
      >
        <IconXMark />
      </Button>
    </div>
  {/each}
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .toaster {
    /* Over everything, centered at the bottom; only the toasts catch the pointer */
    @apply pointer-events-none fixed inset-x-0 bottom-5 z-50 flex flex-col items-center gap-2 px-4;
    /* Centered on the page's content rather than the window, when the side panel (`w-64`) is beside it */
    :global(body:has(.side-panel)) & {
      @apply left-64;
    }
  }

  .toast {
    @apply pointer-events-auto flex max-w-md min-w-72 items-center gap-3 rounded-xl py-2 pr-2 pl-4;
    @apply bg-slate-800/95 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;
    &.error {
      @apply ring-red-400/30;
    }
  }

  .message {
    @apply min-w-0 flex-1 truncate text-sm/5 text-white;
  }

  .action {
    @apply shrink-0 cursor-pointer rounded-md px-2 py-1 text-sm/5 font-semibold text-pink-300 transition-colors hover:bg-white/10 hover:text-pink-200;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }
</style>
