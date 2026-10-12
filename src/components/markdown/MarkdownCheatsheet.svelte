<script lang="ts">
  import IconQuestionMarkCircle from "../common/icons/IconQuestionMarkCircle.svelte";

  const popoverId = $props.id();
  let trigger = $state<HTMLButtonElement>();
  let popover = $state<HTMLElement>();
  let open = $state(false);

  const modifier = navigator.platform.startsWith("Mac") ? "⌘" : "Ctrl+";

  /** Under the button, aligned on its right edge: placed before showing, so it never paints elsewhere. */
  const place = (event: ToggleEvent) => {
    if (event.newState !== "open" || !trigger || !popover) return;
    const { bottom, right } = trigger.getBoundingClientRect();
    popover.style.top = `${bottom + 6}px`;
    popover.style.right = `${document.documentElement.clientWidth - right}px`;
  };
</script>

<!-- Opened and closed (also by clicking outside, or Escape) by the browser, through `popovertarget` -->
<button
  bind:this={trigger}
  type="button"
  class={["trigger", open && "open"]}
  popovertarget={popoverId}
  aria-label="Markdown cheatsheet"
  title="Markdown cheatsheet"
>
  <IconQuestionMarkCircle class="size-5" />
</button>

<div
  bind:this={popover}
  id={popoverId}
  class="cheatsheet"
  popover="auto"
  role="dialog"
  aria-label="Markdown cheatsheet"
  onbeforetoggle={place}
  ontoggle={(event) => (open = event.newState === "open")}
>
  <div class="heading">
    <h2 class="section-title">Markdown</h2>
    <span class="legend">You type → you get</span>
  </div>
  <dl class="rows">
    <div class="row">
      <dt># Title&nbsp;&nbsp;## Section</dt>
      <dd><span class="text-base font-semibold text-white">Title</span> <span class="font-semibold text-white">Section</span></dd>
    </div>
    <div class="row">
      <dt>**bold**&nbsp;&nbsp;_italic_</dt>
      <dd><strong class="font-semibold text-white">bold</strong> <em>italic</em></dd>
    </div>
    <div class="row">
      <dt>~~struck~~</dt>
      <dd><del class="text-slate-500">struck</del></dd>
    </div>
    <div class="row">
      <dt>- item&nbsp;&nbsp;1. item</dt>
      <dd>• item&nbsp;&nbsp;&nbsp;1. item</dd>
    </div>
    <div class="row">
      <dt>- [ ] task&nbsp;&nbsp;- [x] done</dt>
      <dd class="flex items-center gap-2">
        <span class="box" aria-hidden="true"></span>task
        <span class="note">click to check</span>
      </dd>
    </div>
    <div class="row">
      <dt>[text](https://…)</dt>
      <dd>
        <span class="link">text</span>
        <span class="note">opens in your browser</span>
      </dd>
    </div>
    <div class="row">
      <dt>`code`&nbsp;&nbsp;```block```</dt>
      <dd><code class="code">code</code></dd>
    </div>
    <div class="row">
      <dt>&gt; quote</dt>
      <dd class="quote">quote</dd>
    </div>
    <div class="row">
      <dt>| a | b |<br />| - | - |</dt>
      <dd class="note">a table</dd>
    </div>
    <div class="row">
      <dt>---</dt>
      <dd><span class="rule" aria-hidden="true"></span><span class="sr-only">a divider</span></dd>
    </div>
  </dl>
  <p class="shortcuts">
    <kbd>{modifier}B</kbd> bold · <kbd>{modifier}I</kbd> italic · <kbd>Tab</kbd> indent ·
    <kbd>Enter</kbd> continues a list · <kbd>Esc</kbd> leaves the editor
  </p>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  /* As the side panel's "+" button */
  .trigger {
    @apply grid size-8 cursor-pointer place-items-center rounded-lg text-slate-400;
    @apply transition-colors hover:bg-white/10 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
    &.open {
      @apply bg-white/10 text-white;
    }
  }

  /* As the dropdown menus: positioned by script */
  .cheatsheet {
    @apply inset-auto m-0 border-0;
    @apply w-100 max-w-[calc(100vw-1rem)] max-h-[calc(100dvh-5rem)] overflow-y-auto overscroll-contain;
    @apply rounded-xl bg-slate-800/85 p-1 text-sm/5 text-slate-300 shadow-lg ring-1 ring-white/10 backdrop-blur-xl;
    @apply transition-opacity duration-100 ease-out starting:opacity-0;
  }

  .heading {
    @apply flex items-baseline justify-between px-3 pt-1.5 pb-1;
  }

  .section-title {
    @apply text-xs/6 font-semibold tracking-wider text-slate-400 uppercase;
  }

  .legend {
    @apply text-xs text-slate-400;
  }

  .rows {
    @apply flex flex-col;
  }

  .row {
    @apply grid grid-cols-2 items-center gap-3 border-t border-white/5 px-3 py-1.5;
    dt {
      @apply font-mono text-xs text-slate-300;
    }
  }

  .note {
    @apply text-xs text-slate-400;
  }

  .box {
    @apply size-3.5 rounded-[0.25rem] border border-white/15 bg-white/5;
  }

  .link {
    @apply font-medium text-pink-400 underline decoration-pink-400/40 underline-offset-2;
  }

  .code {
    @apply rounded bg-white/5 px-1 py-0.5 font-mono text-xs text-slate-200;
  }

  .quote {
    @apply border-l-2 border-white/15 pl-2.5 text-slate-400 italic;
  }

  .rule {
    @apply block h-px bg-white/15;
  }

  .shortcuts {
    @apply mt-1 border-t border-white/10 px-3 py-2 text-xs/5 text-slate-400;
    kbd {
      @apply rounded px-1.25 py-px font-sans text-slate-300 ring-1 ring-white/15 ring-inset;
    }
  }
</style>
