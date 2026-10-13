import { createContext } from "svelte";

import { computePosition, type Placement } from "./positioning";

/** What receives focus when a menu opens. */
export type MenuFocusTarget = "first" | "last" | "menu" | "none";

const ITEM_SELECTOR = '[role="menuitem"]:not([aria-disabled="true"])';
/** Delay before hovering an item opens its submenu, or closes a sibling's: gives the pointer time to travel diagonally. */
const HOVER_DELAY_MS = 100;
const TYPEAHEAD_RESET_MS = 500;

/** Root menus open below their button; submenus overlap their parent's padding so their first item lines up with the trigger item. */
const MENU_OFFSETS = { offset: 8, alignOffset: 0 };
const SUBMENU_OFFSETS = { offset: 6, alignOffset: -5 };

/**
 * State of one menu level (the root menu or a submenu), shared by its trigger, its popover and its items.
 *
 * The menu is a `popover="auto"` element: the browser handles light dismiss (click outside, Escape) and
 * closing nested submenus, `open` mirrors it through `toggle` events.
 */
export class MenuState {
  readonly parent: MenuState | null;
  readonly triggerId: string;
  readonly menuId: string;

  open = $state(false);
  triggerEl = $state<HTMLElement>();
  menuEl = $state<HTMLElement>();
  /** Set by the menu component. */
  getPlacement: () => Placement = () => "bottom-start";
  /** Set by the menu component: whether clicking one of its items leaves the dropdown open. */
  getKeepOpenOnClick: () => boolean = () => true;
  /** The submenu currently open from this menu, if any. */
  openChild: MenuState | null = null;

  #onOpenChange?: (open: boolean) => void;
  #restoreFocusOnClose = false;
  #hoverTimer: ReturnType<typeof setTimeout> | undefined;
  #typeahead = "";
  #typeaheadTimer: ReturnType<typeof setTimeout> | undefined;

  constructor({
    id,
    parent,
    onOpenChange,
  }: {
    id: string;
    parent: MenuState | null;
    onOpenChange?: (open: boolean) => void;
  }) {
    this.parent = parent;
    this.triggerId = `${id}-trigger`;
    this.menuId = `${id}-menu`;
    this.#onOpenChange = onOpenChange;
  }

  get root(): MenuState {
    return this.parent?.root ?? this;
  }

  /** Opens the menu if needed, then moves focus to `focus`. */
  show(focus: MenuFocusTarget = "menu") {
    const menuEl = this.menuEl;
    if (!menuEl) return;
    if (!menuEl.matches(":popover-open")) {
      // `source` makes the trigger part of the popover for light dismiss, where supported
      menuEl.showPopover({ source: this.triggerEl });
      // Positioned in the same task as the opening, so it never paints at the default position
      this.position();
      this.#setOpen(true);
    }
    this.#focus(focus);
  }

  /** Closes the menu, and its open submenus. */
  hide() {
    if (this.menuEl?.matches(":popover-open")) {
      this.menuEl.hidePopover();
    }
  }

  position() {
    const { menuEl, triggerEl } = this;
    if (!menuEl || !triggerEl) return;
    const { x, y } = computePosition({
      anchor: triggerEl.getBoundingClientRect(),
      floating: menuEl.getBoundingClientRect(),
      viewport: {
        width: document.documentElement.clientWidth,
        height: document.documentElement.clientHeight,
      },
      placement: this.getPlacement(),
      ...(this.parent ? SUBMENU_OFFSETS : MENU_OFFSETS),
    });
    menuEl.style.left = `${x}px`;
    menuEl.style.top = `${y}px`;
  }

  /** Runs `action` after the hover delay, cancelling any pending one. */
  scheduleHover(action: () => void) {
    this.cancelHover();
    this.#hoverTimer = setTimeout(action, HOVER_DELAY_MS);
  }

  cancelHover() {
    clearTimeout(this.#hoverTimer);
  }

  /** Hovered items take the focus, which is what highlights them. */
  hoverItem(item: HTMLElement) {
    if (item.matches(ITEM_SELECTOR)) {
      if (document.activeElement !== item) item.focus({ preventScroll: true });
    } else {
      this.unhoverItem(item);
    }
  }

  unhoverItem(item: HTMLElement) {
    if (document.activeElement === item) {
      this.menuEl?.focus({ preventScroll: true });
    }
  }

  handleBeforeToggle(event: ToggleEvent) {
    if (event.newState === "closed") {
      // Must be checked before the menu hides: focus is lost afterwards
      this.#restoreFocusOnClose = !!this.menuEl?.contains(
        document.activeElement,
      );
    }
  }

  handleToggle(event: ToggleEvent) {
    const open = event.newState === "open";
    this.#setOpen(open);
    if (open) return;

    this.cancelHover();
    const focused = document.activeElement;
    // Return focus to the trigger, unless the user moved it elsewhere (e.g. clicked an input outside)
    if (
      this.#restoreFocusOnClose &&
      (!focused ||
        focused === document.body ||
        this.menuEl?.contains(focused))
    ) {
      this.triggerEl?.focus({ preventScroll: true });
    }
    this.#restoreFocusOnClose = false;
  }

  handleKeydown(event: KeyboardEvent) {
    // Keys pressed in a submenu bubble up to its parent menus
    const target = event.target as HTMLElement;
    if (target.closest('[role="menu"]') !== this.menuEl) return;

    const items = this.#items();
    const index = items.indexOf(document.activeElement as HTMLElement);
    switch (event.key) {
      case "ArrowDown":
        items[(index + 1) % items.length]?.focus();
        break;
      case "ArrowUp":
        items[index <= 0 ? items.length - 1 : index - 1]?.focus();
        break;
      case "Home":
      case "PageUp":
        items[0]?.focus();
        break;
      case "End":
      case "PageDown":
        items.at(-1)?.focus();
        break;
      case "ArrowLeft":
        if (!this.parent) return;
        this.hide();
        break;
      case "Tab": {
        // Items are out of the tab sequence: close everything instead of letting focus escape from a hidden menu
        const root = this.root;
        root.hide();
        root.triggerEl?.focus();
        break;
      }
      default:
        if (
          !this.#searchTypeahead(event, items, index)
        ) {
          return;
        }
    }
    event.preventDefault();
  }

  #setOpen(open: boolean) {
    if (this.open === open) return;
    this.open = open;
    if (this.parent) {
      if (open) {
        this.parent.openChild = this;
      } else if (this.parent.openChild === this) {
        this.parent.openChild = null;
      }
    }
    this.#onOpenChange?.(open);
  }

  /** The enabled items of this menu, excluding the ones of its submenus. */
  #items(): HTMLElement[] {
    const menuEl = this.menuEl;
    if (!menuEl) return [];
    return [...menuEl.querySelectorAll<HTMLElement>(ITEM_SELECTOR)].filter(
      (item) => item.closest('[role="menu"]') === menuEl,
    );
  }

  #focus(target: MenuFocusTarget) {
    switch (target) {
      case "first":
        this.#items()[0]?.focus();
        break;
      case "last":
        this.#items().at(-1)?.focus();
        break;
      case "menu":
        this.menuEl?.focus();
        break;
    }
  }

  /** Focuses the next item starting with the typed characters. */
  #searchTypeahead(
    event: KeyboardEvent,
    items: HTMLElement[],
    index: number,
  ): boolean {
    const { key } = event;
    if (key.length !== 1 || event.ctrlKey || event.metaKey || event.altKey) {
      return false;
    }
    // Space activates the focused item, unless it continues a search
    if (key === " " && !this.#typeahead) return false;

    clearTimeout(this.#typeaheadTimer);
    this.#typeahead += key.toLowerCase();
    this.#typeaheadTimer = setTimeout(
      () => (this.#typeahead = ""),
      TYPEAHEAD_RESET_MS,
    );

    // A new search starts after the focused item; a longer one may still match it
    const start = this.#typeahead.length === 1 ? index + 1 : Math.max(index, 0);
    const match = [...items.slice(start), ...items.slice(0, start)].find(
      (item) =>
        item.textContent?.trim().toLowerCase().startsWith(this.#typeahead),
    );
    match?.focus();
    return true;
  }
}

export const [getMenuContext, setMenuContext] = createContext<MenuState>();
