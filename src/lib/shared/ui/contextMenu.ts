import type { Attachment } from "svelte/attachments";

/**
 * Calls `onOpen` instead of showing the native context menu of the element (right click, Ctrl+click on macOS, or the
 * keyboard's context menu key).
 *
 * With a pointer, `onOpen` is called once its button is released, not pressed: on macOS, `contextmenu` fires on press,
 * and the release that follows would be taken as a click outside a popover menu opened by then, closing it right away.
 *
 * ```svelte
 * <li {@attach contextMenu(() => (menuOpen = true))}>
 * ```
 */
export const contextMenu =
  (onOpen: () => void): Attachment<HTMLElement> =>
  (element) => {
    const isMac = navigator.platform.startsWith("Mac");
    let pressed = false;

    const onPointerDown = (event: PointerEvent) => {
      // Secondary button, or Ctrl + primary button on macOS
      pressed = event.button === 2 || (isMac && event.button === 0 && event.ctrlKey);
    };
    const onPointerUp = () => {
      if (!pressed) return;
      pressed = false;
      onOpen();
    };
    const onContextMenu = (event: MouseEvent) => {
      event.preventDefault();
      // Not from a pointer press: keyboard (e.g. Shift+F10), or a platform firing it after the release (Windows)
      if (!pressed) onOpen();
    };

    element.addEventListener("pointerdown", onPointerDown);
    element.addEventListener("pointerup", onPointerUp);
    element.addEventListener("contextmenu", onContextMenu);
    return () => {
      element.removeEventListener("pointerdown", onPointerDown);
      element.removeEventListener("pointerup", onPointerUp);
      element.removeEventListener("contextmenu", onContextMenu);
    };
  };
