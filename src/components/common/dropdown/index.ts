/**
 * Menu-button dropdown, following the WAI-ARIA menu button pattern. Submenus can be nested at any depth.
 *
 * ```svelte
 * <Dropdown>
 *   <DropdownButton style="outline">Options</DropdownButton>
 *   <DropdownMenu placement="bottom-end">
 *     <DropdownItem onclick={edit}>Edit</DropdownItem>
 *     <DropdownSubmenu label="Move to">
 *       <DropdownItem onclick={() => move("today")}>Today</DropdownItem>
 *       <DropdownItem onclick={() => move("tomorrow")}>Tomorrow</DropdownItem>
 *     </DropdownSubmenu>
 *     <DropdownDivider />
 *     <DropdownItem onclick={remove}>Delete</DropdownItem>
 *   </DropdownMenu>
 * </Dropdown>
 * ```
 *
 * For a button with its own markup and styles, `DropdownTrigger` gives the attributes to spread onto it instead:
 *
 * ```svelte
 * <DropdownTrigger>
 *   {#snippet children(trigger)}
 *     <button {...trigger} class="my-button">Options</button>
 *   {/snippet}
 * </DropdownTrigger>
 * ```
 */
export { default as Dropdown } from "./Dropdown.svelte";
export { default as DropdownButton } from "./DropdownButton.svelte";
export { default as DropdownDivider } from "./DropdownDivider.svelte";
export { default as DropdownItem } from "./DropdownItem.svelte";
export { default as DropdownMenu } from "./DropdownMenu.svelte";
export { default as DropdownSection } from "./DropdownSection.svelte";
export { default as DropdownSubmenu } from "./DropdownSubmenu.svelte";
export { default as DropdownTrigger } from "./DropdownTrigger.svelte";
export type { Placement as DropdownPlacement } from "./positioning";
