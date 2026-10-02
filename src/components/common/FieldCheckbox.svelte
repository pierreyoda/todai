<script lang="ts" module>
  /**
   * [check mark, checked background]
   * Theme variables are spelled out in full so Tailwind detects and emits them.
   */
  const colors = {
    zinc: ["var(--color-white)", "var(--color-zinc-600)"],
    white: ["var(--color-zinc-900)", "var(--color-white)"],
    dark: ["var(--color-white)", "var(--color-zinc-900)"],
    red: ["var(--color-white)", "var(--color-red-600)"],
    orange: ["var(--color-white)", "var(--color-orange-500)"],
    amber: ["var(--color-amber-950)", "var(--color-amber-400)"],
    yellow: ["var(--color-yellow-950)", "var(--color-yellow-300)"],
    lime: ["var(--color-lime-950)", "var(--color-lime-300)"],
    green: ["var(--color-white)", "var(--color-green-600)"],
    emerald: ["var(--color-white)", "var(--color-emerald-600)"],
    teal: ["var(--color-white)", "var(--color-teal-600)"],
    cyan: ["var(--color-cyan-950)", "var(--color-cyan-300)"],
    sky: ["var(--color-white)", "var(--color-sky-500)"],
    blue: ["var(--color-white)", "var(--color-blue-600)"],
    indigo: ["var(--color-white)", "var(--color-indigo-500)"],
    violet: ["var(--color-white)", "var(--color-violet-500)"],
    purple: ["var(--color-white)", "var(--color-purple-500)"],
    fuchsia: ["var(--color-white)", "var(--color-fuchsia-500)"],
    pink: ["var(--color-white)", "var(--color-pink-500)"],
    rose: ["var(--color-white)", "var(--color-rose-500)"],
  } as const satisfies Record<string, readonly [string, string]>;

  export type FieldCheckboxColor = keyof typeof colors;
</script>

<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type FieldCheckboxProps = Omit<
    HTMLInputAttributes,
    "type" | "checked" | "indeterminate"
  > & {
    label: string;
    /** Keeps the label for screen readers only. */
    hideLabel?: boolean;
    description?: string;
    checked?: boolean;
    color?: FieldCheckboxColor;
    class?: string;
  };

  let {
    label,
    hideLabel = false,
    description,
    checked = $bindable(false),
    color = "zinc",
    class: extraFieldClass = "",
    ...inputProps
  }: FieldCheckboxProps = $props();

  const uid = $props.id();
  const descriptionId = `${uid}-description`;
  let inputId = $derived(inputProps.id ?? `${uid}-input`);

  let [checkColor, checkedBg] = $derived(colors[color]);
</script>

<div
  class={["field", extraFieldClass]}
  style:--checkbox-check={checkColor}
  style:--checkbox-checked-bg={checkedBg}
>
  <span class="control">
    <input
      type="checkbox"
      bind:checked
      aria-describedby={description ? descriptionId : undefined}
      {...inputProps}
      id={inputId}
    />
    <span class="box" aria-hidden="true">
      <svg viewBox="0 0 14 14" fill="none">
        <path class="check-icon" d="M3 8L6 11L11 3.5" />
        <path class="indeterminate-icon" d="M3 7H11" />
      </svg>
    </span>
  </span>
  <label for={inputId} class={["label", hideLabel && "sr-only"]}>{label}</label>
  {#if description}
    <p id={descriptionId} class="description">{description}</p>
  {/if}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .field {
    @apply grid grid-cols-[1.125rem_1fr] gap-x-4 gap-y-1 sm:grid-cols-[1rem_1fr];
  }

  .control {
    @apply relative col-start-1 row-start-1 mt-0.75 inline-flex sm:mt-1;
  }

  .label {
    @apply col-start-2 row-start-1 cursor-pointer text-base/6 text-white select-none sm:text-sm/6;
  }

  .field:has(.description) .label {
    @apply font-medium;
  }

  .description {
    @apply col-start-2 row-start-2 text-base/6 text-slate-400 sm:text-sm/6;
  }

  /* The native input sits invisibly on top of the box so it keeps receiving clicks and focus. */
  input {
    @apply absolute inset-0 z-10 m-0 size-full cursor-pointer appearance-none opacity-0;
    @apply disabled:cursor-not-allowed;
  }

  .box {
    /* Layout */
    @apply relative isolate flex size-4.5 items-center justify-center rounded-[0.3125rem] sm:size-4;
    /* Unchecked */
    @apply border border-white/15 bg-white/5;
    /* Inner highlight shadow, only shown when checked */
    @apply after:absolute after:-inset-px after:hidden after:rounded-[0.3125rem] after:shadow-[inset_0_1px_--theme(--color-white/15%)];
    /* Forced colors mode */
    @apply forced-colors:[--checkbox-check:HighlightText] forced-colors:[--checkbox-checked-bg:Highlight];
  }

  input:hover + .box {
    @apply border-white/30;
  }

  input:checked + .box,
  input:indeterminate + .box {
    @apply border-white/5 bg-(--checkbox-checked-bg) after:block;
  }

  input:focus-visible + .box {
    @apply outline-2 outline-offset-2 outline-blue-500;
  }

  input:disabled + .box {
    @apply border-white/20 bg-white/2.5 opacity-50 [--checkbox-check:color-mix(in_oklab,var(--color-white)_50%,transparent)] after:hidden;
    @apply forced-colors:[--checkbox-check:Highlight];
  }

  input:disabled:is(:checked, :indeterminate) + .box {
    @apply bg-(--checkbox-checked-bg);
  }

  svg {
    @apply size-4 stroke-(--checkbox-check) stroke-2 opacity-0 sm:size-3.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  input:is(:checked, :indeterminate) + .box svg {
    @apply opacity-100;
  }

  .indeterminate-icon,
  input:indeterminate + .box .check-icon {
    @apply opacity-0;
  }

  input:indeterminate + .box .indeterminate-icon {
    @apply opacity-100;
  }
</style>
