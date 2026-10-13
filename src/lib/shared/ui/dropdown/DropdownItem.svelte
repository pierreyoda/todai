<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ClassValue } from "svelte/elements";

  import { getMenuContext, type MenuState } from "./menu.svelte";

  type DropdownItemProps = {
    /** Renders a link instead of a button. */
    href?: string;
    disabled?: boolean;
    /** Called before the whole dropdown closes, if it does (see `keepOpen`). */
    onclick?: (event: MouseEvent) => void;
    /**
     * Whether clicking the item leaves the dropdown open. Defaults to its menu's `keepOpenOnClick`.
     *
     * Must be `false` for items opening a modal: WebKit's `showModal()` hides the menu without a `toggle` event,
     * which leaves it open in state, and impossible to reopen.
     */
    keepOpen?: boolean;
    /** Leading `<svg>`, sized and colored by the item. */
    icon?: Snippet;
    class?: ClassValue;
    /** Internal: the submenu this item opens, provided by `DropdownSubmenu`. */
    submenu?: MenuState;
    children: Snippet;
  };
  const {
    href,
    disabled = false,
    onclick,
    keepOpen,
    icon,
    class: extraClass,
    submenu,
    children,
  }: DropdownItemProps = $props();

  const menu = getMenuContext();

  let element = $state<HTMLElement>();
  $effect(() => {
    if (submenu) submenu.triggerEl = element;
  });

  const isLink = $derived(!!href && !disabled && !submenu);

  function handleClick(event: MouseEvent) {
    if (disabled) {
      event.preventDefault();
      return;
    }
    if (submenu) {
      // `detail` is 0 for clicks triggered by the keyboard (Enter, Space)
      submenu.show(event.detail === 0 ? "first" : "none");
      return;
    }
    onclick?.(event);
    if (!(keepOpen ?? menu.getKeepOpenOnClick())) menu.root.hide();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!submenu || disabled || event.key !== "ArrowRight") return;
    event.preventDefault();
    submenu.show("first");
  }

  function handlePointerEnter() {
    menu.scheduleHover(() => {
      if (submenu && !disabled) {
        // Opening a submenu closes its open siblings (popover stack)
        submenu.show("none");
      } else {
        menu.openChild?.hide();
      }
    });
  }
</script>

<svelte:element
  this={isLink ? "a" : "button"}
  bind:this={element}
  href={isLink ? href : undefined}
  type={isLink ? undefined : "button"}
  id={submenu?.triggerId}
  role="menuitem"
  tabindex="-1"
  aria-disabled={disabled || undefined}
  aria-haspopup={submenu ? "menu" : undefined}
  aria-expanded={submenu ? submenu.open : undefined}
  aria-controls={submenu?.menuId}
  class={["item", extraClass]}
  onclick={handleClick}
  onkeydown={handleKeydown}
  onpointerenter={handlePointerEnter}
  onpointermove={(event: PointerEvent) =>
    menu.hoverItem(event.currentTarget as HTMLElement)}
  onpointerleave={(event: PointerEvent) =>
    menu.unhoverItem(event.currentTarget as HTMLElement)}
>
  {#if icon}
    <span class="icon" aria-hidden="true">{@render icon()}</span>
  {/if}
  <span class="label">{@render children()}</span>
  {#if submenu}
    <svg
      class="icon"
      viewBox="0 0 16 16"
      fill="currentColor"
      aria-hidden="true"
    >
      <path
        fill-rule="evenodd"
        d="M6.22 4.22a.75.75 0 0 1 1.06 0l3.25 3.25a.75.75 0 0 1 0 1.06l-3.25 3.25a.75.75 0 0 1-1.06-1.06L8.94 8 6.22 5.28a.75.75 0 0 1 0-1.06Z"
        clip-rule="evenodd"
      />
    </svg>
  {/if}
</svelte:element>

<style lang="postcss">
  @reference "tailwindcss";

  .item {
    /* Base */
    @apply flex w-full cursor-default items-center gap-3 rounded-lg px-3.5 py-2.5 text-left select-none sm:px-3 sm:py-1.5;
    /* Typography */
    @apply text-base/6 text-white sm:text-sm/6;
    /* Highlighted: hovered or focused with the keyboard */
    @apply outline-hidden focus:bg-blue-500;
    /* Submenu open while the pointer is in it */
    @apply aria-expanded:not-focus:bg-white/10;
    /* Disabled */
    @apply aria-disabled:opacity-50;
    /* Forced colors mode */
    @apply forced-colors:focus:bg-[Highlight] forced-colors:focus:text-[HighlightText];
  }

  .icon {
    @apply size-5 shrink-0 text-slate-400 sm:size-4;
    /* The icon `<svg>` comes from the parent's scope */
    > :global(svg) {
      @apply size-full;
    }
  }

  .item:focus .icon {
    @apply text-white;
  }

  .label {
    @apply min-w-0 flex-1 truncate;
  }
</style>
