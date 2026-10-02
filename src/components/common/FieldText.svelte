<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type FieldTextProps = Omit<HTMLInputAttributes, "type" | "value"> & {
    label: string;
    /** Keeps the label for screen readers only. */
    hideLabel?: boolean;
    value?: string;
    class?: string;
  };

  let {
    label,
    hideLabel = false,
    value = $bindable(""),
    class: extraFieldClass = "",
    ...inputProps
  }: FieldTextProps = $props();
</script>

<label class={["field", extraFieldClass]}>
  <span class={["label", hideLabel && "sr-only"]}>{label}</span>
  <span class="control">
    <input type="text" bind:value {...inputProps} />
  </span>
</label>

<style lang="postcss">
  @reference "tailwindcss";

  .field {
    @apply flex w-full flex-col gap-1.5;
  }

  .label {
    @apply text-sm font-medium text-slate-300;
  }

  /* Focus ring drawn on a wrapper so it sits outside the input border. */
  .control {
    @apply relative block w-full;
    @apply after:pointer-events-none after:absolute after:inset-0 after:rounded-lg after:ring-transparent after:ring-inset focus-within:after:ring-2 focus-within:after:ring-blue-500;
    @apply has-disabled:opacity-50;
  }

  input {
    /* Layout */
    @apply relative block w-full appearance-none rounded-lg px-3 py-1.5;
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
    @apply disabled:cursor-not-allowed disabled:border-white/15 disabled:bg-white/2.5;
  }
</style>
