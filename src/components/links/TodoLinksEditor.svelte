<script lang="ts">
  import {
    createAddTodoLinkMutation,
    createRemoveTodoLinkMutation,
  } from "../../client/mutations";
  import type { Todo } from "../../client/types";
  import { linkKindName } from "../../utils/links";
  import { openLink } from "../../utils/openLink";
  import Button from "../common/Button.svelte";
  import FieldText from "../common/FieldText.svelte";
  import IconXMark from "../common/icons/IconXMark.svelte";
  import LinkKindIcon from "./LinkKindIcon.svelte";

  /** The links of a todo, added and removed right away: unlike its other fields, they're not submitted. */
  type TodoLinksEditorProps = {
    todo: Todo;
  };
  const { todo }: TodoLinksEditorProps = $props();

  const titleId = $props.id();
  let url = $state("");
  const addLink = createAddTodoLinkMutation(() => todo);
  const removeLink = createRemoveTodoLinkMutation(() => todo);

  const onsubmit = (event: SubmitEvent) => {
    event.preventDefault();
    if (!url.trim() || addLink.isPending) return;
    addLink.mutate(url, { onSuccess: () => (url = "") });
  };
</script>

<section class="links" aria-labelledby={titleId}>
  <h3 id={titleId} class="title">Links</h3>
  {#if todo.links.length > 0}
    <ul class="list">
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
  {/if}
  <!-- A form of its own, so that Enter adds the link rather than submitting the todo -->
  <form class="add" {onsubmit}>
    <FieldText
      label="Link to add"
      hideLabel
      placeholder="Paste a pull request, an issue or any link"
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

  .title {
    /* Like the fields' labels */
    @apply text-sm font-medium text-slate-300;
  }

  .list {
    @apply flex flex-col divide-y divide-white/5 rounded-xl bg-white/3 ring-1 ring-white/10;
  }

  .row {
    @apply flex items-center gap-2.5 py-2 pr-2 pl-3;
  }

  .icon {
    @apply shrink-0;

    > :global(svg) {
      @apply text-slate-400;
    }
  }

  .text {
    @apply flex min-w-0 flex-1 flex-col rounded-md;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500;
  }

  .label {
    @apply truncate text-sm/5 font-medium text-white;

    .text:hover & {
      @apply underline underline-offset-2;
    }
  }

  .detail {
    @apply truncate text-xs/4 text-slate-400;
  }

  .remove {
    @apply grid size-7 shrink-0 cursor-pointer place-items-center rounded-md text-slate-400;
    @apply transition-colors hover:bg-white/10 hover:text-white disabled:cursor-not-allowed disabled:opacity-50;
    @apply focus:not-focus-visible:outline-hidden focus-visible:outline-2 focus-visible:outline-blue-500;
  }

  .add {
    @apply mt-1 flex items-start gap-2;
  }

  .error {
    @apply text-sm/5 text-red-400;
  }
</style>
