<script lang="ts">
  import {
    createMutation,
    createQuery,
    useQueryClient,
  } from "@tanstack/svelte-query";

  import { invokeClient } from "../../client";
  import type { Tag, Todo } from "../../client/types";
  import {
    activeWorkspaceQueryOptions,
    invalidateTodosOf,
  } from "../../client/queries";
  import { createToggleTodoMutation } from "../../client/mutations";
  import EditableText from "../common/EditableText.svelte";
  import FieldCheckbox from "../common/FieldCheckbox.svelte";
  import TodoDropdownMenu from "./TodoDropdownMenu.svelte";
  import Button from "../common/Button.svelte";
  import IconEllipsisVertical from "../common/icons/IconEllipsisVertical.svelte";
  import IconClock from "../common/icons/IconClock.svelte";
  import TodoLinkChip from "../links/TodoLinkChip.svelte";
  import IconCheckCircle from "../common/icons/IconCheckCircle.svelte";
  import { doneSuggestion } from "../../utils/linkStatuses";
  import { formatEstimates } from "../../utils";

  type TodoLineItemProps = {
    item: Todo;
    tags: readonly Tag[];
  };
  const { item, tags }: TodoLineItemProps = $props();

  // Its GitHub links show their status through the workspace's connection, if any
  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const connected = $derived(!!activeWorkspace.data?.githubLogin);

  // Resolved from the tags list, so they follow tag renames without refetching todos
  const itemTags = $derived(tags.filter((tag) => item.tagIds.includes(tag.id)));

  // Follows the server state, but is checked/unchecked right away on click.
  let completed = $derived(item.completed);
  const toggleCompleted = createToggleTodoMutation(
    () => item,
    () => (completed = item.completed),
  );
  /** Its merged pull request or completed issue, if any, which suggests marking it as done. */
  const doneBecause = $derived(
    connected && activeWorkspace.data?.linksSuggestDone
      ? doneSuggestion({ completed, links: item.links })
      : undefined,
  );

  const queryClient = useQueryClient();

  // svelte-ignore state_referenced_locally: edited locally, then saved
  let editedTitle = $state(item.title);
  const updateTitle = createMutation(() => ({
    mutationFn: (title: string) =>
      invokeClient({
        name: "update_todo",
        args: { params: { id: item.id, title } },
      }),
    onSuccess: () => {
      return invalidateTodosOf(queryClient, item.day);
    },
    onError: () => {
      editedTitle = item.title;
    },
  }));
  const saveTitle = (title: string) => {
    editedTitle = title;
    // Also called when confirming the unchanged title
    if (title !== item.title) updateTitle.mutate(title);
  };

  let openedDropdownMenu = $state(false);
  let menuButton = $state<HTMLButtonElement>();
</script>

<li class={["todo-item", item.completed && "completed"]}>
  <div class="flex items-center">
    <FieldCheckbox
      label="Completed?"
      hideLabel
      bind:checked={completed}
      onchange={(event) => toggleCompleted.mutate(event.currentTarget.checked)}
    />
    <div class="flex flex-col gap-1">
      <EditableText
        as="h3"
        bind:value={() => editedTitle, saveTitle}
        label={`Title of todo "${item.title}"`}
        class="text-sm font-medium text-white wrap-break-word"
      />
      {#if itemTags.length > 0 || item.links.length > 0}
        <div class="flex items-center gap-4 overflow-x-auto">
          {#each itemTags as itemTag (itemTag.id)}
            <div class="flex items-center gap-1">
              <div
                class={["rounded-full w-2 h-2"]}
                style:background-color={itemTag.color}
              ></div>
              <span class="text-white text-xs">{itemTag.name}</span>
            </div>
          {/each}
          {#if item.links.length > 0}
            {#if itemTags.length > 0}
              <span class="links-divider" aria-hidden="true"></span>
            {/if}
            <ul class="links" aria-label="Links">
              {#each item.links as link (link.id)}
                <li class="flex">
                  <TodoLinkChip {link} todo={item} {connected} />
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </div>
  </div>
  <div class="shrink-0">
    <div class="flex items-center gap-2">
      {#if doneBecause}
        <!-- Only suggested: a todo is never completed on its own -->
        <Button
          style="plain"
          size="xs"
          class="suggest-done"
          title={`${doneBecause.link.label} was ${doneBecause.state}`}
          disabled={toggleCompleted.isPending}
          onclick={() => {
            completed = true;
            toggleCompleted.mutate(true);
          }}
        >
          <IconCheckCircle />
          {doneBecause.state === "merged" ? "Merged" : "Completed"} · mark as done
        </Button>
      {/if}
      {#if item.estimate}
        <div class="flex items-center gap-1 text-white">
          {#if item.estimate.unit === "minutes"}
            <IconClock />
          {/if}
          <span class="text-sm">{formatEstimates([item.estimate])}</span>
        </div>
      {/if}
      <Button
        style="plain"
        bind:ref={menuButton}
        aria-haspopup="menu"
        aria-expanded={openedDropdownMenu}
        onclick={() => (openedDropdownMenu = true)}
      >
        <IconEllipsisVertical />
      </Button>
      <TodoDropdownMenu
        bind:open={openedDropdownMenu}
        anchor={menuButton}
        {item}
        {tags}
      />
    </div>
  </div>
</li>

<style lang="postcss">
  @reference "tailwindcss";

  /* A `Button`, restyled through its line as it's another component's element: more specific than its own rules */
  .todo-item :global(.suggest-done) {
    @apply inline-flex h-6.5 shrink-0 items-center gap-1.5 rounded-md border-0 px-2.5 py-0;
    @apply bg-purple-400/16 text-xs/4 font-semibold text-purple-200 ring-1 ring-purple-400/35 ring-inset;
    @apply transition-colors disabled:cursor-default disabled:opacity-60 focus-visible:outline-offset-0;

    /* Highlighted when hovered, even disabled, but not when pressed */
    &:enabled:active {
      @apply bg-purple-400/16;
    }
    &:enabled:hover,
    &:disabled:hover {
      @apply hover:bg-purple-400/25;
    }

    > :global(svg) {
      @apply size-3.5;
    }
  }

  .todo-item {
    @apply flex items-center justify-between gap-1 px-4 py-3 rounded-lg;
    @apply border border-slate-600 bg-slate-800;
    @apply transition-colors hover:border-slate-400 hover:bg-slate-600;
    &.completed {
      @apply border-gray-700 bg-gray-900 hover:bg-gray-700;
    }
  }

  /* Between the tags and the links, which are told apart at a glance */
  .links-divider {
    @apply -mx-1 h-3.5 w-px shrink-0 bg-white/10;
  }

  .links {
    @apply flex shrink-0 items-center gap-1.5;
  }
</style>
