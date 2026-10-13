<script lang="ts">
  import { useIsFetching } from "@tanstack/svelte-query";

  import type { Todo } from "../../model/types";
  import type { TodoLink } from "../model/types";
  import { linkStatusKeys } from "../api/queries";
  import { itemState, linkStatus } from "../model/statuses";
  import LinkChip from "./LinkChip.svelte";
  import LinkPopover from "./LinkPopover.svelte";

  type TodoLinkChipProps = {
    link: TodoLink;
    todo: Todo;
    /** Whether the active workspace is connected to GitHub: its pull requests and issues then show their status. */
    connected: boolean;
  };
  const { link, todo, connected }: TodoLinkChipProps = $props();

  // Shown as loading while fetched for the first time
  const refreshing = useIsFetching({ queryKey: linkStatusKeys.all });
  const withDetails = $derived(connected && link.github !== undefined);
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
  <LinkPopover bind:open {link} {todo} anchor={chip} />
{:else}
  <LinkChip {link} />
{/if}
