<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type FieldTextProps = Omit<HTMLInputAttributes, "type" | "value"> & {
    label: string;
    /** Keeps the label for screen readers only. */
    hideLabel?: boolean;
    value?: string;
  };

  let { label, hideLabel = false, value = $bindable(""), ...inputProps }: FieldTextProps = $props();
</script>

<label class="field">
  <span class={["label", hideLabel && "sr-only"]}>{label}</span>
  <input type="text" bind:value {...inputProps} />
</label>

<style lang="postcss">
  @reference "tailwindcss";

  .field {
    @apply flex flex-col gap-1;
  }

  .label {
    @apply text-sm font-medium text-slate-700;
  }

  input {
    @apply w-full px-3 py-2 bg-white rounded shadow outline-none;
    @apply focus:ring-2 focus:ring-slate-600;
  }
</style>
