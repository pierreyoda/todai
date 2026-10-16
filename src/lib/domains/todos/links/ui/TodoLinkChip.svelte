<script lang="ts">
import { useIsFetching } from "@tanstack/svelte-query";

import type { Todo } from "../../model/types";
import { linkStatusKeys } from "../api/queries";
import { itemState } from "../model/github";
import { linkProvider } from "../model/links";
import { linkStatus } from "../model/statuses";
import type { TodoLink } from "../model/types";
import GitHubLinkPopover from "./GitHubLinkPopover.svelte";
import LinkChip from "./LinkChip.svelte";

type TodoLinkChipProps = {
  link: TodoLink;
  todo: Todo;
  /** Whether the active workspace is connected to the link's provider, if any: its item then shows its status. */
  connected: boolean;
};
const { link, todo, connected }: TodoLinkChipProps = $props();

// Shown as loading while fetched for the first time
const refreshing = useIsFetching({ queryKey: linkStatusKeys.all });
const provider = $derived(linkProvider(link));
const withDetails = $derived(connected && provider !== undefined);
const status = $derived.by(() => {
  if (!withDetails) return null;
  return linkStatus(link) ?? (refreshing.current > 0 ? "loading" : null);
});
const linkState = $derived(
  withDetails && link.snapshot ? itemState(link.snapshot) : undefined,
);

let open = $state(false);
let chip = $state<HTMLElement>();
</script>

{#if withDetails}
  <LinkChip
    {link}
    state={linkState}
    {status}
    active={open}
    bind:ref={chip}
    onclick={() => (open = !open)}
  />
  {#if provider === "github"}
    <GitHubLinkPopover bind:open {link} {todo} anchor={chip} />
  {/if}
{:else}
  <LinkChip {link} />
{/if}
