<script lang="ts">
  import { untrack, type Snippet } from "svelte";

  import { MenuState, setMenuContext } from "./menu.svelte";

  type DropdownProps = {
    /** Bindable: lets the parent open or close the menu programmatically. */
    open?: boolean;
    /**
     * Without a `DropdownButton` or `DropdownTrigger`, e.g. when only opened programmatically: the element the menu is
     * positioned against, and gives the focus back to when closing.
     */
    anchor?: HTMLElement;
    /** A `DropdownButton` or `DropdownTrigger` (unless `anchor` is set), possibly nested, and a `DropdownMenu`. */
    children: Snippet;
  };
  let { open = $bindable(false), anchor, children }: DropdownProps = $props();

  const id = $props.id();
  const menu = new MenuState({
    id,
    parent: null,
    onOpenChange: (value) => (open = value),
  });
  setMenuContext(menu);

  $effect.pre(() => {
    if (anchor) menu.triggerEl = anchor;
  });

  $effect(() => {
    const shouldOpen = open;
    // Also re-runs once the menu element is mounted, for menus initially open
    if (!menu.menuEl) return;
    untrack(() => {
      // Opening from the button already happened and focused the right element
      if (shouldOpen === menu.open) return;
      if (shouldOpen) menu.show();
      else menu.hide();
    });
  });
</script>

{@render children()}
