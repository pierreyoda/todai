<script lang="ts" module>
  import type { Attachment } from "svelte/attachments";
  import type { HTMLButtonAttributes } from "svelte/elements";

  /** Attributes making a button toggle the menu, to spread onto it. */
  export type DropdownTriggerAttributes = Pick<
    HTMLButtonAttributes,
    | "type"
    | "id"
    | "aria-haspopup"
    | "aria-expanded"
    | "aria-controls"
    | "onpointerdown"
    | "onclick"
    | "onkeydown"
  > & { [key: symbol]: Attachment<HTMLButtonElement> };
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import { createAttachmentKey } from "svelte/attachments";

  import { getMenuContext } from "./menu.svelte";

  type DropdownTriggerProps = {
    /** The button, with its own markup and styles: must spread the given attributes onto its element. */
    children: Snippet<[DropdownTriggerAttributes]>;
  };
  const { children }: DropdownTriggerProps = $props();

  const menu = getMenuContext();

  let wasOpenOnPointerDown = false;

  const onpointerdown = () => (wasOpenOnPointerDown = menu.open);

  const onclick = (event: MouseEvent) => {
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
  };

  const onkeydown = (event: KeyboardEvent) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      menu.show(event.key === "ArrowDown" ? "first" : "last");
    }
  };

  const ref = createAttachmentKey();
  const attachTrigger: Attachment<HTMLButtonElement> = (element) => {
    menu.triggerEl = element;
    return () => {
      // Unmounted (e.g. replaced while editing): the menu mustn't stay anchored to a detached element
      if (menu.triggerEl === element) menu.triggerEl = undefined;
    };
  };

  // Handlers and attachment are created once: new ones would be rebound each time the menu opens or closes
  const trigger = $derived<DropdownTriggerAttributes>({
    type: "button",
    id: menu.triggerId,
    "aria-haspopup": "menu",
    "aria-expanded": menu.open,
    "aria-controls": menu.menuId,
    onpointerdown,
    onclick,
    onkeydown,
    [ref]: attachTrigger,
  });
</script>

{@render children(trigger)}
