<script lang="ts">
  import type { ButtonProps } from "./Button.svelte";
  import Button from "./Button.svelte";

  type ConfirmButtonProps = ButtonProps;
  const { children, onclick, ...props }: ConfirmButtonProps = $props();

  let confirm = $state(false);
</script>

<Button
  onclick={(event) => {
    if (confirm) {
      confirm = false;
      onclick?.(event);
    } else confirm = true;
  }}
  {...props}
>
  {#if confirm}
    Confirm
  {:else}
    {@render children?.()}
  {/if}
</Button>
