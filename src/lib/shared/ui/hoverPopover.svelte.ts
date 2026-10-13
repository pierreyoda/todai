import { computePosition } from "./dropdown/positioning";

/** How long hovering takes to show the panel, and leaving it to hide it. */
const HOVER_OPEN_MS = 150;
const HOVER_CLOSE_MS = 200;

/**
 * A marker's panel (e.g. a todo's links after its title, or a day's note), shown under it while it's hovered, or once
 * it's clicked. Hovering the panel keeps it shown: the panel is part of the group holding both, though shown on top.
 *
 * A manual popover, so that showing it leaves any other one open (e.g. a menu): closed by Escape or by clicking
 * elsewhere, as it's not light-dismissed.
 *
 * Must be created during component initialization. Then:
 * - bind `button` to the marker's button, and `panel` to its `popover="manual"` panel;
 * - set `onpointerenter` and `onpointerleave` on the group holding both, and `onclick` on the button.
 */
export class HoverPopover {
  /** Whether the panel is shown: e.g. set to `false` once a link in it is followed. */
  open = $state(false);
  button = $state<HTMLButtonElement>();
  panel = $state<HTMLElement>();

  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor() {
    $effect(() => () => clearTimeout(this.#timer));

    $effect(() => {
      const { panel } = this;
      if (!panel) return;
      const shown = panel.matches(":popover-open");
      if (this.open && !shown) {
        panel.showPopover();
        // Positioned in the same task as the opening, so it never paints at the default position
        this.#position();
      } else if (!this.open && shown) {
        panel.hidePopover();
      }
    });

    $effect(() => {
      if (!this.open) return;
      const onpointerdown = (event: PointerEvent) => {
        const target = event.target as Node;
        if (!this.button?.contains(target) && !this.panel?.contains(target)) this.open = false;
      };
      const onkeydown = (event: KeyboardEvent) => {
        if (event.key !== "Escape") return;
        const hadFocus = this.panel?.contains(document.activeElement);
        this.open = false;
        if (hadFocus) this.button?.focus();
      };
      document.addEventListener("pointerdown", onpointerdown, true);
      document.addEventListener("keydown", onkeydown);
      window.addEventListener("resize", this.#position);
      window.addEventListener("scroll", this.#position, { capture: true, passive: true });
      return () => {
        document.removeEventListener("pointerdown", onpointerdown, true);
        document.removeEventListener("keydown", onkeydown);
        window.removeEventListener("resize", this.#position);
        window.removeEventListener("scroll", this.#position, { capture: true });
      };
    });
  }

  /** On the group holding the button and the panel. */
  readonly onpointerenter = (event: PointerEvent) => {
    if (event.pointerType === "mouse") this.#later(() => (this.open = true), HOVER_OPEN_MS);
  };

  /** On the group holding the button and the panel. */
  readonly onpointerleave = (event: PointerEvent) => {
    if (event.pointerType === "mouse") this.#later(() => (this.open = false), HOVER_CLOSE_MS);
  };

  /** On the button: clicked once hovered, the panel is kept shown; with the keyboard, shown or hidden. */
  readonly onclick = (event: MouseEvent) => {
    clearTimeout(this.#timer);
    this.open = event.detail > 0 ? true : !this.open;
  };

  #later(action: () => void, delay: number) {
    clearTimeout(this.#timer);
    this.#timer = setTimeout(action, delay);
  }

  #position = () => {
    const { button, panel } = this;
    if (!button || !panel) return;
    const { x, y } = computePosition({
      anchor: button.getBoundingClientRect(),
      floating: panel.getBoundingClientRect(),
      viewport: {
        width: document.documentElement.clientWidth,
        height: document.documentElement.clientHeight,
      },
      placement: "bottom-start",
      offset: 4,
    });
    panel.style.left = `${x}px`;
    panel.style.top = `${y}px`;
  };
}
