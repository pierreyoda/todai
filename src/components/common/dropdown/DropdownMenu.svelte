<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassValue } from "svelte/elements";

  import {
    getMenuContext,
    setMenuContext,
    type MenuState,
  } from "./menu.svelte";
  import type { Placement } from "./positioning";

  type DropdownMenuProps = {
    /** Side of the button the menu opens on, flipped when it would overflow the viewport. */
    placement?: Placement;
    /**
     * Whether clicking one of its items leaves the dropdown open; items can override it with `keepOpen`.
     * Defaults to the parent menu's for submenus, `true` otherwise.
     */
    keepOpenOnClick?: boolean;
    class?: ClassValue;
    /** Internal: the submenu state, provided by `DropdownSubmenu`. */
    menu?: MenuState;
    children: Snippet;
  };
  const {
    placement = "bottom-start",
    keepOpenOnClick,
    class: extraClass,
    menu: submenu,
    children,
  }: DropdownMenuProps = $props();

  // svelte-ignore state_referenced_locally: a menu level never changes
  const menu = submenu ?? getMenuContext();
  menu.getPlacement = () => placement;
  menu.getKeepOpenOnClick = () =>
    keepOpenOnClick ?? menu.parent?.getKeepOpenOnClick() ?? true;
  // Items belong to this menu level
  setMenuContext(menu);

  // Keep the menu anchored to its trigger
  $effect(() => {
    if (!menu.open) return;
    const reposition = () => menu.position();
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, {
      capture: true,
      passive: true,
    });
    return () => {
      window.removeEventListener("resize", reposition);
      window.removeEventListener("scroll", reposition, { capture: true });
    };
  });
</script>

<div
  bind:this={menu.menuEl}
  id={menu.menuId}
  popover="auto"
  role="menu"
  tabindex="-1"
  aria-labelledby={menu.triggerId}
  class={["menu", extraClass]}
  onbeforetoggle={(event) => menu.handleBeforeToggle(event)}
  ontoggle={(event) => menu.handleToggle(event)}
  onkeydown={(event) => menu.handleKeydown(event)}
  onpointerenter={() => menu.parent?.cancelHover()}
>
  {@render children()}
</div>

<style lang="postcss">
  @reference "tailwindcss";

  .menu {
    /* Reset popover defaults: the menu is positioned by script */
    @apply inset-auto m-0 border-0;
    /* Sizing: scrolls when taller than the viewport */
    @apply w-max max-w-[calc(100vw-1rem)] min-w-48 max-h-[calc(100dvh-1rem)] overflow-y-auto overscroll-contain;
    /* Surface */
    @apply isolate rounded-xl bg-slate-800/80 p-1 text-white shadow-lg ring-1 ring-white/10 backdrop-blur-xl outline-hidden;
    /* Opening transition */
    @apply transition-opacity duration-100 ease-out starting:opacity-0;
    /* Forced colors mode */
    @apply forced-colors:outline;
  }
</style>
