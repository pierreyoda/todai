<script lang="ts">
  import type { HTMLTextareaAttributes } from "svelte/elements";

  type FieldTextAreaProps = Omit<HTMLTextareaAttributes, "value"> & {
    label: string;
    /** Keeps the label for screen readers only. */
    hideLabel?: boolean;
    value?: string;
    /** Monospace, e.g. to write Markdown. */
    mono?: boolean;
    /**
     * Takes its container's height rather than `rows` lines, without resize handle: the container must be a flex column
     * of bounded height.
     */
    fill?: boolean;
    class?: string;
    /** Bindable: the inner `<textarea>` element. */
    ref?: HTMLTextAreaElement;
  };

  const textareaId = $props.id();
  let {
    label,
    hideLabel = false,
    value = $bindable(""),
    mono = false,
    fill = false,
    class: extraFieldClass = "",
    ref = $bindable(),
    rows = 3,
    ...textareaProps
  }: FieldTextAreaProps = $props();
</script>

<div class={["field", fill && "fill", extraFieldClass]}>
  <label for={textareaId} class={["label", hideLabel && "sr-only"]}>{label}</label>
  <span class="control">
    <textarea
      {...textareaProps}
      bind:this={ref}
      id={textareaId}
      {rows}
      bind:value
      class={[mono && "mono"]}
    ></textarea>
  </span>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .field {
    @apply flex w-full flex-col gap-1.5;
    &.fill {
      @apply min-h-0 flex-1;
    }
  }

  .label {
    @apply text-sm font-medium text-slate-300;
  }

  /* Focus ring drawn on a wrapper so it sits outside the textarea border, as in `FieldText`. */
  .control {
    @apply relative block w-full;
    @apply after:pointer-events-none after:absolute after:inset-0 after:rounded-lg after:ring-transparent after:ring-inset focus-within:after:ring-2 focus-within:after:ring-blue-500;
    @apply has-disabled:opacity-50;
    .fill & {
      @apply flex min-h-0 flex-1;
    }
  }

  textarea {
    /* Layout: block, so that no baseline gap is left under it */
    @apply relative block w-full resize-y appearance-none rounded-lg px-3 py-1.5;
    /* Typography */
    @apply text-base/6 text-white placeholder:text-slate-500 sm:text-sm/6;
    /* Border */
    @apply border border-white/10 hover:border-white/20;
    /* Background */
    @apply bg-white/5;
    /* Hide default focus styles, the wrapper ring replaces them */
    @apply focus:outline-hidden;
    /* Invalid state */
    @apply aria-invalid:border-red-600 aria-invalid:hover:border-red-600;
    /* Disabled state */
    @apply disabled:cursor-not-allowed disabled:resize-none disabled:border-white/15 disabled:bg-white/2.5;

    &.mono {
      @apply font-mono text-[0.8125rem]/5.5;
    }

    /* A writing area rather than a field: roomier */
    .fill & {
      @apply h-full resize-none px-3.5 py-3;
    }
  }
</style>
