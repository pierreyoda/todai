<script lang="ts">
  import type { ComponentProps } from "svelte";

  import Button from "../Button.svelte";
  import { getMenuContext } from "./menu.svelte";

  /** Same props as `Button`: clicking it toggles the menu. */
  type DropdownButtonProps = Omit<
    ComponentProps<typeof Button>,
    "onclick" | "ref"
  >;
  const props: DropdownButtonProps = $props();

  const menu = getMenuContext();

  let wasOpenOnPointerDown = false;
</script>

<Button
  {...props}
  bind:ref={
    () => menu.triggerEl as HTMLButtonElement | undefined,
    (element) => (menu.triggerEl = element)
  }
  id={menu.triggerId}
  aria-haspopup="menu"
  aria-expanded={menu.open}
  aria-controls={menu.menuId}
  onpointerdown={() => (wasOpenOnPointerDown = menu.open)}
  onclick={(event) => {
    // `detail` is 0 for clicks triggered by the keyboard (Enter, Space)
    const fromKeyboard = event.detail === 0;
    // Pressing the button may light-dismiss the menu before `click` fires: don't reopen it then
    const wasOpen = fromKeyboard ? menu.open : wasOpenOnPointerDown;
    wasOpenOnPointerDown = false;
    if (wasOpen) {
      menu.hide();
    } else {
      menu.show(fromKeyboard ? "first" : "menu");
    }
  }}
  onkeydown={(event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      menu.show(event.key === "ArrowDown" ? "first" : "last");
    }
  }}
/>
