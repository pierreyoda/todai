<script lang="ts">
  import { tick, untrack } from "svelte";

  type EditableTextProps = {
    as: "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span";
    value: string;
    /** Accessible label of the text field, visually hidden. */
    label: string;
    class?: string;
    /** Bindable: whether the text field is shown, e.g. to start editing from a menu rather than a click on the text. */
    editing?: boolean;
  };

  let {
    as,
    value = $bindable(),
    label,
    class: extraClass = "",
    editing = $bindable(false),
  }: EditableTextProps = $props();

  let draft = $state("");
  let trigger = $state<HTMLButtonElement>();

  const invalid = $derived(draft.trim() === "");

  // Started from the text or by the parent: either way, the draft starts from the current text
  $effect.pre(() => {
    if (editing) draft = untrack(() => value);
  });

  const startEditing = () => (editing = true);

  const stopEditing = async (save: boolean) => {
    // The input can blur right after Enter/Escape already closed it.
    if (!editing) return;
    editing = false;
    if (save && !invalid) value = draft.trim();
    await tick();
    trigger?.focus();
  };

  const onkeydown = (event: KeyboardEvent) => {
    if (event.key === "Enter") {
      event.preventDefault();
      stopEditing(true);
    } else if (event.key === "Escape") {
      event.preventDefault();
      stopEditing(false);
    }
  };

  const focusAtEnd = (input: HTMLInputElement) => {
    input.focus();
    input.setSelectionRange(input.value.length, input.value.length);
  };
</script>

<!-- The text and its field share the element, so both get the same typography -->
<svelte:element this={as} class={["editable-text", extraClass]}>
  {#if editing}
    <span class="field">
      <!-- Invisible copy of the text, sizing the field to its content -->
      <span class="sizer" aria-hidden="true">{draft}</span>
      <input
        type="text"
        size="1"
        bind:value={draft}
        aria-label={label}
        aria-invalid={invalid}
        onblur={() => stopEditing(true)}
        {onkeydown}
        {@attach focusAtEnd}
      />
    </span>
  {:else}
    <button type="button" bind:this={trigger} onclick={startEditing}>
      {value}
    </button>
  {/if}
</svelte:element>

<style lang="postcss">
  @reference "tailwindcss";

  /* Left-aligned whatever the surrounding alignment, like its field */
  .editable-text {
    @apply min-w-0 text-left;
  }

  /* Same box for the text and its field, offset so the text stays aligned with its surroundings */
  button,
  .sizer,
  input {
    @apply -mx-1 rounded px-1;
  }

  /* As wide as the text, so entering edit mode doesn't change the layout; grows while typing */
  .field {
    @apply inline-grid max-w-full grid-cols-[minmax(0,auto)] align-top;
  }

  .sizer,
  input {
    @apply col-start-1 row-start-1;
  }

  .sizer {
    @apply invisible whitespace-pre;
  }

  button {
    @apply cursor-text text-left;
  }

  input {
    /* Layout */
    @apply w-[calc(100%+(--spacing(2)))] min-w-0 appearance-none;
    /* Typography: inherited from the surrounding text */
    @apply [font:inherit] tracking-[inherit] text-inherit;
    /* Border and background: transparent. */
    /* Focus: the ring replaces the default outline */
    @apply focus:ring-0 focus:outline-hidden;
    /* Invalid state */
    @apply aria-invalid:ring-red-600 aria-invalid:hover:ring-red-600;
  }
</style>
