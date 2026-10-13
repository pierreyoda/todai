<script lang="ts">
import type { ClassValue } from "svelte/elements";
import {
  Dropdown,
  DropdownItem,
  DropdownMenu,
  DropdownSection,
  DropdownTrigger,
} from "#shared/ui/dropdown/index.ts";
import IconChevronUpDown from "#shared/ui/icons/IconChevronUpDown.svelte";
import type { Workspace } from "../model/types";
import WorkspaceAvatar from "./WorkspaceAvatar.svelte";

type WorkspacePickerProps = {
  workspaces: readonly Workspace[];
  selected: Workspace;
  onPicked: (workspace: Workspace) => void;
  /** Applied to the button opening it. */
  class?: ClassValue;
};
const {
  workspaces,
  selected,
  onPicked,
  class: extraClass,
}: WorkspacePickerProps = $props();
</script>

{#snippet status({ available, isActive }: Workspace)}
  {#if !available}
    <span class="status unavailable">Unavailable</span>
  {:else if isActive}
    <span class="status">Active</span>
  {/if}
{/snippet}

<Dropdown>
  <DropdownTrigger>
    {#snippet children(trigger)}
      <button
        {...trigger}
        class={["picker", extraClass]}
        aria-label={`Workspace: ${selected.name}`}
      >
        <WorkspaceAvatar name={selected.name} size="sm" />
        <span class="name">{selected.name}</span>
        {@render status(selected)}
        <IconChevronUpDown class="text-slate-400" />
      </button>
    {/snippet}
  </DropdownTrigger>
  <DropdownMenu keepOpenOnClick={false}>
    <DropdownSection heading="Workspaces">
      {#each workspaces as workspace (workspace.id)}
        <DropdownItem onclick={() => onPicked(workspace)}>
          <span class="option">
            <WorkspaceAvatar name={workspace.name} size="sm" />
            <span class="name">{workspace.name}</span>
            {@render status(workspace)}
          </span>
        </DropdownItem>
      {/each}
    </DropdownSection>
  </DropdownMenu>
</Dropdown>

<style lang="postcss">
  @reference "tailwindcss";

  .picker {
    /* Layout: like an outline button, tighter on the avatar's side */
    @apply inline-flex min-w-0 items-center gap-2 rounded-lg border border-white/15 py-[calc(--spacing(1.5)-1px)] pr-[calc(--spacing(2.5)-1px)] pl-[calc(--spacing(2)-1px)];
    /* Typography */
    @apply text-sm/6 font-semibold text-white;
    /* States */
    @apply cursor-pointer transition-colors hover:bg-white/5 aria-expanded:bg-white/5;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
  }

  .option {
    @apply flex min-w-0 items-center gap-2;
  }

  .name {
    @apply min-w-0 truncate;
  }

  /* Takes the room of a widened button (see `class`), pushing its status and chevron to the end */
  .picker .name {
    @apply flex-1 text-left;
  }

  .status {
    @apply shrink-0 text-xs font-medium text-slate-400;
    &.unavailable {
      @apply text-red-400;
    }
  }
</style>
