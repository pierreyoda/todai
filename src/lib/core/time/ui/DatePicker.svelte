<script lang="ts">
  import type { Day } from "../days";
  import { formatShortDay } from "../days";
  import Button from "#shared/ui/Button.svelte";
  import DatePickerModal from "./DatePickerModal.svelte";
  import IconCalendar from "#shared/ui/icons/IconCalendar.svelte";

  /** A button showing the selected `day`, which opens a month calendar in a modal to pick another one. */
  type DayPickerProps = {
    /** Bindable: the picked day, `null` until one is. */
    day?: Day | null;
    /** Title of the modal, also shown by the button when `day` is `null`. */
    label?: string;
    disabled?: boolean;
    class?: string;
  };
  let {
    day = $bindable(null),
    label,
    disabled = false,
    class: extraClass,
  }: DayPickerProps = $props();

  let show = $state(false);
</script>

<Button
  style="outline"
  {disabled}
  aria-haspopup="dialog"
  onclick={() => (show = true)}
  class={extraClass}
>
  <IconCalendar class="self-center" />
  {day ? formatShortDay(day) : label}
</Button>
<DatePickerModal bind:show {day} {label} onPick={(picked) => (day = picked)} />
