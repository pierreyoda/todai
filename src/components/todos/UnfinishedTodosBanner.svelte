<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { slide } from "svelte/transition";

  import {
    activeWorkspaceQueryOptions,
    unfinishedTodosQueryOptions,
  } from "../../client/queries";
  import { createMoveTodosMutation } from "../../client/mutations";
  import type { Day, Tag, Todo } from "../../client/types";
  import { UNFINISHED_TODOS_LOOKBACK_DAYS } from "../../constants";
  import { addDays, formatDay } from "#core/time";
  import Button from "#shared/ui/Button.svelte";
  import IconCalendar from "#shared/ui/icons/IconCalendar.svelte";
  import IconSun from "#shared/ui/icons/IconSun.svelte";
  import IconXMark from "#shared/ui/icons/IconXMark.svelte";

  /** Offers to move to `day` the todos left unfinished in the days before it: all at once, or picked one by one. */
  type UnfinishedTodosBannerProps = {
    day: Day;
    tags: readonly Tag[];
  };
  const { day, tags }: UnfinishedTodosBannerProps = $props();

  const unfinished = createQuery(() => unfinishedTodosQueryOptions(day));
  const todos = $derived(unfinished.data ?? []);

  const activeWorkspace = createQuery(() => activeWorkspaceQueryOptions);
  const workspaceId = $derived(activeWorkspace.data?.id);

  /** Hidden until the next day, per workspace, on this machine only: holds the day it was hidden on. */
  const storageKey = (workspaceId: string) =>
    `todai:unfinished-todos-dismissed:${workspaceId}`;
  const readDismissedOn = (workspaceId: string) => {
    try {
      return localStorage.getItem(storageKey(workspaceId));
    } catch {
      return null;
    }
  };
  /** The workspaces it was hidden for since the page was opened. */
  let dismissedNow = $state<string[]>([]);
  const dismissed = $derived(
    !workspaceId ||
      dismissedNow.includes(workspaceId) ||
      readDismissedOn(workspaceId) === day,
  );
  const dismiss = () => {
    if (!workspaceId) return;
    dismissedNow = [...dismissedNow, workspaceId];
    try {
      localStorage.setItem(storageKey(workspaceId), day);
    } catch {
      // Only until the page is opened again, then
    }
  };

  const shown = $derived(!dismissed && todos.length > 0);

  let expanded = $state(false);
  const listId = $props.id();

  const moveTodos = createMoveTodosMutation(() => day);

  /** Most recent day first; todos keep their display order. */
  const todosByDay = $derived.by(() => {
    const byDay = new Map<Day, Todo[]>();
    for (const todo of todos) {
      byDay.set(todo.day, [...(byDay.get(todo.day) ?? []), todo]);
    }
    return [...byDay].reverse();
  });
  const yesterday = $derived(addDays(day, -1));

  const tagsOf = (todo: Todo) =>
    tags.filter((tag) => todo.tagIds.includes(tag.id));
</script>

{#if shown}
  <aside
    class="banner"
    aria-label="Unfinished todos"
    transition:slide={{ duration: 200 }}
  >
    <div class="header">
      <span class="icon"><IconCalendar /></span>
      <p class="text">
        <span class="title">
          {todos.length} unfinished {todos.length === 1 ? "todo" : "todos"}
        </span>
        <span class="detail">
          from the last {UNFINISHED_TODOS_LOOKBACK_DAYS} days
        </span>
      </p>
      <Button
        style="plain"
        size="sm"
        aria-expanded={expanded}
        aria-controls={expanded ? listId : undefined}
        onclick={() => (expanded = !expanded)}
      >
        {expanded ? "Hide" : "Review"}
        <svg
          class={["chevron", expanded && "open"]}
          viewBox="0 0 16 16"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fill-rule="evenodd"
            d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"
            clip-rule="evenodd"
          />
        </svg>
      </Button>
      <Button
        color="zinc"
        size="sm"
        disabled={moveTodos.isPending}
        onclick={() => moveTodos.mutate(todos)}
      >
        Move all to today
      </Button>
      <Button
        style="plain"
        size="xs"
        class="shrink-0 text-slate-400! hover:text-white!"
        aria-label="Hide until tomorrow"
        title="Hide until tomorrow"
        onclick={dismiss}
      >
        <IconXMark />
      </Button>
    </div>
    {#if moveTodos.isError}
      <p class="error" role="alert">
        Could not move them: {String(moveTodos.error)}
      </p>
    {/if}
    {#if expanded}
      <div id={listId} class="days" transition:slide={{ duration: 200 }}>
        {#each todosByDay as [todoDay, dayTodos] (todoDay)}
          <section>
            <h3 class="day">
              {todoDay === yesterday ? "Yesterday" : formatDay(todoDay)}
            </h3>
            <ul>
              {#each dayTodos as todo (todo.id)}
                {@const todoTags = tagsOf(todo)}
                <li class="todo">
                  <span class="todo-title" title={todo.title}>{todo.title}</span>
                  {#if todoTags.length > 0}
                    <ul class="tags" aria-label="Tags">
                      {#each todoTags as tag (tag.id)}
                        <li class="tag">
                          <span class="tag-color" style:background-color={tag.color}></span>
                          {tag.name}
                        </li>
                      {/each}
                    </ul>
                  {/if}
                  <Button
                    style="plain"
                    size="xs"
                    class="move"
                    aria-label={`Move “${todo.title}” to today`}
                    disabled={moveTodos.isPending}
                    onclick={() => moveTodos.mutate([todo])}
                  >
                    <IconSun />
                    Move to today
                  </Button>
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      </div>
    {/if}
  </aside>
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .banner {
    @apply mx-2 mb-2 flex flex-col rounded-xl bg-white/3 ring-1 ring-white/10;
  }

  .header {
    @apply flex items-center gap-2 px-3.5 py-2.5;
  }

  .icon {
    @apply mr-1 grid size-8 shrink-0 place-items-center rounded-full bg-pink-400/12 text-pink-400;

    > :global(svg) {
      @apply size-4;
    }
  }

  .text {
    @apply flex min-w-0 flex-1 flex-wrap items-baseline gap-x-1.5 text-sm/5;
  }

  .title {
    @apply font-medium text-white;
  }

  .detail {
    @apply text-slate-400;
  }

  .chevron {
    @apply size-4 self-center transition-transform motion-reduce:transition-none;
    &.open {
      @apply rotate-180;
    }
  }

  .error {
    @apply px-3.5 pb-2.5 text-sm text-red-400;
  }

  /* Scrolls on its own past a few days, so that the day's todos stay in view */
  .days {
    @apply flex max-h-72 flex-col gap-2 overflow-y-auto border-t border-white/10 px-2 py-2;
  }

  .day {
    @apply px-2 pb-0.5 text-xs font-medium text-slate-400;
  }

  .todo {
    @apply flex min-w-0 items-center gap-3 rounded-md py-1 pr-1 pl-2 transition-colors hover:bg-white/5;
  }

  .todo-title {
    @apply min-w-0 flex-1 truncate text-sm text-white;
  }

  .tags {
    @apply flex shrink-0 items-center gap-2;
  }

  .tag {
    @apply flex items-center gap-1 text-xs text-slate-400;
  }

  .tag-color {
    @apply size-2 rounded-full;
  }

  /* Muted until its row is hovered: the list reads as a list first */
  .todo :global(.move) {
    @apply shrink-0 text-slate-400 transition-colors;
    @apply flex items-center;
  }
  .todo:hover :global(.move) {
    @apply text-white;
  }
</style>
