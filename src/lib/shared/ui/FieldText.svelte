<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLInputAttributes } from "svelte/elements";

  type FieldTextProps = Omit<HTMLInputAttributes, "type" | "value"> & {
    label: string;
    /** Keeps the label for screen readers only. */
    hideLabel?: boolean;
    type?: "text" | "search";
    value?: string;
    class?: string;
    /** Bindable: the inner `<input>` element. */
    ref?: HTMLInputElement;
    /** Decorative content at the start of the input, e.g. an icon: clicks go through to the input. */
    leading?: Snippet;
    /** Content at the end of the input, e.g. a button. Replaces the native clear button of search inputs. */
    trailing?: Snippet;
  };

  const inputId = $props.id();
  let {
    label,
    hideLabel = false,
    type = "text",
    value = $bindable(""),
    class: extraFieldClass = "",
    ref = $bindable(),
    leading,
    trailing,
    ...inputProps
  }: FieldTextProps = $props();
</script>

<!-- The label only wraps its text: the trailing content (e.g. a button) would otherwise be part of the input's name -->
<div class={["field", extraFieldClass]}>
  <label for={inputId} class={["label", hideLabel && "sr-only"]}>{label}</label>
  <span class="control">
    <input
      {...inputProps}
      bind:this={ref}
      id={inputId}
      {type}
      bind:value
      class={[leading && "with-leading", trailing && "with-trailing"]}
    />
    {#if leading}
      <span class="leading">{@render leading()}</span>
    {/if}
    {#if trailing}
      <span class="trailing">{@render trailing()}</span>
    {/if}
  </span>
</div>

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

    /* Room for the leading and trailing content, over the input */
    &.with-leading {
      @apply pl-9;
    }
    &.with-trailing {
      @apply pr-9;
      &::-webkit-search-cancel-button {
        @apply appearance-none;
      }
    }
  }

  .leading,
  .trailing {
    @apply absolute inset-y-0 flex items-center;
  }

  .leading {
    @apply pointer-events-none left-0 pl-3 text-slate-500;
  }

  .trailing {
    @apply right-0 pr-1.5;
  }
</style>
