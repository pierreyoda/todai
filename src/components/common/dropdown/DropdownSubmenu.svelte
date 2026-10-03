<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassValue } from "svelte/elements";

  import DropdownItem from "./DropdownItem.svelte";
  import DropdownMenu from "./DropdownMenu.svelte";
  import { getMenuContext, MenuState } from "./menu.svelte";

  type DropdownSubmenuProps = {
    /** Text of the item opening the submenu. */
    label: string;
    /** Leading `<svg>` of the item opening the submenu. */
    icon?: Snippet;
    disabled?: boolean;
    /** Applied to the submenu panel. */
    class?: ClassValue;
    /** The submenu items, which can include nested `DropdownSubmenu`s. */
    children: Snippet;
  };
  const {
    label,
    icon,
    disabled = false,
    class: extraClass,
    children,
  }: DropdownSubmenuProps = $props();

  const id = $props.id();
  const menu = new MenuState({ id, parent: getMenuContext() });
</script>

<DropdownItem {icon} {disabled} submenu={menu}>{label}</DropdownItem>
<!-- Rendered inside the parent menu: nested popovers stay open together, and close with it -->
<DropdownMenu {menu} placement="right-start" class={extraClass}>
  {@render children()}
</DropdownMenu>
