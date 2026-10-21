<script lang="ts">
type TodoBadgeProps = {
  /** Its urgent flag, or that it's started (in the board's "Doing" column). */
  kind: "urgent" | "doing";
  /** E.g. when it was started. */
  title?: string;
};
const { kind, title }: TodoBadgeProps = $props();

const LABELS = { urgent: "Urgent", doing: "Doing" } as const;
</script>

<span class={["badge", kind]} {title}>{LABELS[kind]}</span>

<style lang="postcss">
  @reference "tailwindcss";

  /* As tall as its links' chips, with a dot of its color before its word */
  .badge {
    @apply inline-flex h-5.5 shrink-0 items-center gap-1.5 rounded-md px-2 ring-1 ring-inset;
    @apply text-xs/4 font-semibold whitespace-nowrap;
    @apply before:size-1.5 before:rounded-full before:bg-current;

    &.urgent {
      @apply bg-rose-400/14 text-rose-300 ring-rose-400/35;
    }
    &.doing {
      @apply bg-sky-400/14 text-sky-300 ring-sky-400/35;
    }
  }
</style>
