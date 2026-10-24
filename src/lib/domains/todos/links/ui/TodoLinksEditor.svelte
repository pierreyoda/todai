<script lang="ts">
import Button from "#shared/ui/Button.svelte";
import FieldText from "#shared/ui/FieldText.svelte";
import IconXMark from "#shared/ui/icons/IconXMark.svelte";
import type { Todo } from "../../model/types";
import {
  createAddTodoLinksMutation,
  createRemoveTodoLinkMutation,
} from "../api/mutations";
import { openLink } from "../api/openLink";
import { formatLinksCount, linkItemName, linkKindName } from "../model/links";
import LinkKindIcon from "./LinkKindIcon.svelte";

/**
 * The links of a todo, added and removed right away: unlike its other fields, they're not submitted. As tall however
 * many there are, two lines of them, scrolling past them.
 */
type TodoLinksEditorProps = {
  todo: Todo;
};
const { todo }: TodoLinksEditorProps = $props();

const titleId = $props.id();
let url = $state("");
const addLink = createAddTodoLinksMutation(() => todo);
const removeLink = createRemoveTodoLinkMutation(() => todo);

const onsubmit = (event: SubmitEvent) => {
  event.preventDefault();
  if (!url.trim() || addLink.isPending) return;
  addLink.mutate([url], { onSuccess: () => (url = "") });
};

let list = $state<HTMLUListElement>();
/** Whether links are hidden below the box: the last line shown then fades out, as macOS hides scrollbars. */
let moreBelow = $state(false);
const updateMoreBelow = () => {
  moreBelow =
    !!list && list.scrollTop + list.clientHeight < list.scrollHeight - 1;
};
// As links are added or removed, and once laid out: its modal mounts it closed, 0px tall
$effect(() => {
  void todo.links.length;
  updateMoreBelow();
  if (!list) return;
  const observer = new ResizeObserver(updateMoreBelow);
  observer.observe(list);
  return () => observer.disconnect();
});
</script>

<section class="links" aria-labelledby={titleId}>
  <div class="head">
    <div class="heading">
      <h3 id={titleId} class="title">Links</h3>
      {#if todo.links.length > 0}
        <span class="count">{formatLinksCount(todo.links.length)}</span>
      {/if}
    </div>
    <!-- A form of its own, so that Enter adds the link rather than submitting the todo -->
    <form class="add" {onsubmit}>
      <FieldText
        label="Link to add"
        hideLabel
        placeholder="Paste a pull request, an issue, a Notion page or any link"
        autocomplete="off"
        spellcheck={false}
        bind:value={url}
        class="min-w-0 flex-1"
      />
      <Button
        type="submit"
        style="outline"
        disabled={!url.trim() || addLink.isPending}
      >
        Add
      </Button>
    </form>
  </div>
  <div class="box">
    {#if todo.links.length > 0}
      <ul
        bind:this={list}
        class={["list", moreBelow && "more-below"]}
        onscroll={updateMoreBelow}
      >
        {#each todo.links as link (link.id)}
          <li class="row">
            <span class="icon"><LinkKindIcon kind={link.kind} /></span>
            <a
              class="text"
              href={link.url}
              title={`${link.url}: opens in your browser`}
              onclick={(event) => {
                event.preventDefault();
                openLink(link.url);
              }}
            >
              <span class="label">{link.label}</span>
              <span class="detail">
                {#if link.github}
                  {linkKindName(link.kind)} · {link.github.owner}/{link.github.repo}
                {:else if link.notion}
                  {linkItemName(link)}
                {:else}
                  {link.url}
                {/if}
              </span>
            </a>
            <button
              type="button"
              class="remove"
              title="Remove"
              aria-label={`Remove the link to ${link.label}`}
              disabled={removeLink.isPending}
              onclick={() => removeLink.mutate(link)}
            >
              <IconXMark />
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="empty">No links yet</p>
    {/if}
  </div>
  {#if addLink.isError}
    <p class="error" role="alert">{String(addLink.error)}</p>
  {/if}
  {#if removeLink.isError}
    <p class="error" role="alert">{String(removeLink.error)}</p>
  {/if}
</section>

<style lang="postcss">
  @reference "tailwindcss";

  .links {
    @apply flex flex-col gap-1.5;
  }

  /* The title on the left, the field adding a link on the right */
  .head {
    @apply flex items-center gap-3 py-3 justify-between;
  }

  .heading {
    @apply flex shrink-0 items-baseline gap-2;
  }

  .title {
    /* Like the fields' labels */
    @apply text-sm font-medium text-slate-300;
  }

  /* Like the description's syntax */
  .count {
    @apply text-xs/4 text-slate-500;
  }

  .add {
    @apply pl-4 flex min-w-0 flex-1 items-start gap-2;
  }

  /* Two lines of 36px and their divider */
  .box {
    @apply h-48 shrink-0 overflow-hidden rounded-xl bg-white/3 ring-1 ring-white/10;
  }

  .list {
    @apply flex h-full flex-col divide-y divide-white/5 overflow-y-auto;

    &.more-below {
      mask-image: linear-gradient(to bottom, #000 calc(100% - 1.75rem), rgb(0 0 0 / 0.15));
    }
  }

  .empty {
    @apply grid h-full place-items-center text-sm/5 text-slate-500;
  }

  /* As tall as its remove button */
  .row {
    @apply flex shrink-0 items-center gap-2.5 py-1 pr-2 pl-3;
  }

  .icon {
    @apply shrink-0;

    > :global(svg) {
      @apply text-slate-400;
    }
  }

  /* Its label, then what it is: the label keeps room for it */
  .text {
    @apply flex min-w-0 flex-1 items-baseline gap-2.5 rounded-md;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
  }

  .label {
    @apply max-w-[55%] shrink-0 truncate text-sm/5 font-medium text-white;

    .text:hover & {
      @apply underline underline-offset-2;
    }
  }

  .detail {
    @apply min-w-0 truncate text-xs/4 text-slate-400;
  }

  .remove {
    @apply grid size-7 shrink-0 cursor-pointer place-items-center rounded-md text-slate-400;
    @apply transition-colors hover:bg-white/10 hover:text-white disabled:cursor-not-allowed disabled:opacity-50;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }

  .error {
    @apply text-sm/5 text-red-400;
  }
</style>
