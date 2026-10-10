<script lang="ts">
  import FieldText from "./FieldText.svelte";
  import IconMagnifyingGlass from "./icons/IconMagnifyingGlass.svelte";
  import IconXMark from "./icons/IconXMark.svelte";

  type SearchFieldProps = {
    /** Bindable: the query. */
    value?: string;
    /** Accessible label, also shown as the placeholder. */
    label: string;
    /** Whether Cmd+F (Ctrl+F outside macOS) focuses it: for the main search of a page only. */
    focusShortcut?: boolean;
    class?: string;
  };
  let {
    value = $bindable(""),
    label,
    focusShortcut = false,
    class: extraClass,
  }: SearchFieldProps = $props();

  let input = $state<HTMLInputElement>();

  const isMac = navigator.platform.startsWith("Mac");

  const clear = () => {
    value = "";
    input?.focus();
  };

  const onkeydown = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    // Clears the query first, then leaves the field
    if (value) value = "";
    else input?.blur();
  };

  const onWindowKeydown = (event: KeyboardEvent) => {
    if (!focusShortcut) return;
    const modifier = isMac ? event.metaKey : event.ctrlKey;
    if (!modifier || event.altKey || event.shiftKey) return;
    if (event.key.toLowerCase() !== "f") return;
    event.preventDefault();
    input?.focus();
    input?.select();
  };
</script>

<svelte:window onkeydown={onWindowKeydown} />

<FieldText
  type="search"
  bind:value
  bind:ref={input}
  {label}
  hideLabel
  placeholder={label}
  autocomplete="off"
  spellcheck={false}
  class={extraClass}
  {onkeydown}
>
  {#snippet leading()}
    <IconMagnifyingGlass />
  {/snippet}
  {#snippet trailing()}
    {#if value}
      <button
        type="button"
        class="clear"
        title="Clear"
        aria-label="Clear the search"
        onclick={clear}
      >
        <IconXMark />
      </button>
    {:else if focusShortcut}
      <kbd class="shortcut" aria-hidden="true">{isMac ? "⌘F" : "Ctrl+F"}</kbd>
    {/if}
  {/snippet}
</FieldText>

<style lang="postcss">
  @reference "tailwindcss";

  .clear {
    /* Layout */
    @apply grid size-6 cursor-pointer place-items-center rounded-md;
    /* States */
    @apply text-slate-400 transition-colors hover:bg-white/10 hover:text-white;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-blue-500;
  }

  .shortcut {
    @apply mr-1 rounded border border-white/10 px-1.5 font-sans text-xs/5 text-slate-500;
  }
</style>
