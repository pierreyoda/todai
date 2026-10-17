<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import Button from "#shared/ui/Button.svelte";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import Modal from "#shared/ui/Modal.svelte";
import {
  createDeleteNotionDoneRuleMutation,
  createSetNotionDoneRuleMutation,
} from "../api/mutations";
import {
  notionColumnsQueryOptions,
  notionDoneRulesQueryOptions,
} from "../api/queries";
import {
  columnKindName,
  DEFAULT_RULE,
  preselectedColumn,
  preselectedOptions,
  ruleFor,
} from "../model/rules";
import NotionColorDot from "./NotionColorDot.svelte";

/** Sets which values of which column of a Notion database mean its pages are done. */
type NotionDoneRuleModalProps = {
  /** Bindable. */
  show: boolean;
  /** The database's data source, in Notion's terms. */
  dataSourceId: string;
  /** Its title as known, until Notion tells it. */
  databaseTitle?: string;
};
let {
  show = $bindable(),
  dataSourceId,
  databaseTitle,
}: NotionDoneRuleModalProps = $props();

const formId = $props.id();
const columnsQuery = createQuery(() => ({
  ...notionColumnsQueryOptions(dataSourceId),
  enabled: show,
}));
const rules = createQuery(() => notionDoneRulesQueryOptions);
const rule = $derived(
  rules.data?.find((rule) => rule.dataSourceId === dataSourceId),
);
const close = () => (show = false);
const save = createSetNotionDoneRuleMutation({ onSuccess: close });
const reset = createDeleteNotionDoneRuleMutation({ onSuccess: close });

const columns = $derived(columnsQuery.data?.columns ?? []);
const title = $derived(
  columnsQuery.data?.title || databaseTitle || "this database",
);

/** The column picked, by its property's ID. */
let pickedId = $state<string | null>(null);
/** The options meaning done, by their ID. */
let doneIds = $state<string[]>([]);
const picked = $derived(
  columns.find((column) => column.propertyId === pickedId),
);

// Fresh each time it's opened: the database's columns may have changed in Notion
$effect(() => {
  if (!show) {
    pickedId = null;
    return;
  }
  untrack(() => {
    save.reset();
    reset.reset();
  });
});
// Preselected once the columns are known: the rule's, if any
$effect(() => {
  if (!show || columns.length === 0) return;
  untrack(() => {
    if (pickedId !== null) return;
    const column = preselectedColumn(columns, rule);
    pickedId = column?.propertyId ?? null;
    doneIds = column ? preselectedOptions(column, rule) : [];
  });
});

const pick = (propertyId: string) => {
  pickedId = propertyId;
  const column = columns.find((column) => column.propertyId === propertyId);
  doneIds = column ? preselectedOptions(column, rule) : [];
};

const next = $derived(picked && ruleFor(dataSourceId, title, picked, doneIds));
const busy = $derived(save.isPending || reset.isPending);

const onsubmit = (event: SubmitEvent) => {
  event.preventDefault();
  if (next && !busy) save.mutate(next);
};
</script>

<Modal bind:show>
  {#snippet header()}
    When is a page of {title} done?
  {/snippet}
  <div class="body">
    <p class="intro">
      By default, a page is done once {DEFAULT_RULE}. Pick another column, and
      the values meaning done: the todos linked to its pages then offer “mark as
      done” once they hold one. Nothing is ever completed on its own.
    </p>

    {#if columnsQuery.isPending}
      <p class="status" role="status">Asking Notion for its columns…</p>
    {:else if columnsQuery.isError}
      <ErrorBanner
        title="Could not ask Notion for its columns"
        error={columnsQuery.error}
      />
    {:else if columns.length === 0}
      <p class="status">
        {title} has no status, select, multi-select or checkbox column: its
        pages can only be done as Notion's Status tells.
      </p>
    {:else}
      <form id={formId} class="form" {onsubmit}>
        <fieldset class="group">
          <legend class="legend">Column</legend>
          {#each columns as column (column.propertyId)}
            <label class={["choice", pickedId === column.propertyId && "picked"]}>
              <input
                type="radio"
                name="notion-done-column"
                value={column.propertyId}
                checked={pickedId === column.propertyId}
                onchange={() => pick(column.propertyId)}
              />
              <span class="name">{column.name}</span>
              <span class="tag">{columnKindName(column.kind)}</span>
            </label>
          {/each}
        </fieldset>

        {#if picked?.kind === "checkbox"}
          <p class="note">Its pages are done once {picked.name} is checked.</p>
        {:else if picked && picked.options.length === 0}
          <p class="note">{picked.name} has no options yet.</p>
        {:else if picked}
          <fieldset class="group">
            <legend class="legend">Done when {picked.name} is</legend>
            <div class="options">
              {#each picked.options as option (option.id)}
                <label class="option">
                  <input
                    type="checkbox"
                    value={option.id}
                    bind:group={doneIds}
                  />
                  <NotionColorDot color={option.color} />
                  <span class="name">{option.name}</span>
                </label>
              {/each}
            </div>
          </fieldset>
        {/if}
      </form>
    {/if}

    {#if save.isError}
      <ErrorBanner
        title="Could not save the rule"
        error={save.error}
        onDismiss={() => save.reset()}
      />
    {/if}
    {#if reset.isError}
      <ErrorBanner
        title="Could not reset the rule"
        error={reset.error}
        onDismiss={() => reset.reset()}
      />
    {/if}
  </div>
  {#snippet actions()}
    {#if rule}
      <Button
        style="plain"
        class="mr-auto"
        disabled={busy}
        onclick={() => reset.mutate(dataSourceId)}
      >
        Use Notion's Status
      </Button>
    {/if}
    <Button style="plain" onclick={close}>Cancel</Button>
    <Button type="submit" form={formId} disabled={!next || busy}>Save</Button>
  {/snippet}
</Modal>

<style lang="postcss">
  @reference "tailwindcss";

  .body {
    @apply flex flex-col gap-4;
  }

  .intro {
    @apply text-sm/6 text-slate-400;
  }

  .status,
  .note {
    @apply text-sm/5 text-slate-400;
  }

  .form {
    @apply flex flex-col gap-4;
  }

  .group {
    @apply m-0 flex flex-col gap-2 border-0 p-0;
  }

  .legend {
    @apply mb-1.5 p-0 text-sm/5 font-medium text-slate-300;
  }

  .choice {
    @apply flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2.5 ring-1 ring-white/10 transition-colors hover:bg-white/5;

    &.picked {
      @apply bg-white/6 ring-pink-400/60;
    }

    input {
      @apply size-4 accent-pink-500;
    }
  }

  .name {
    @apply min-w-0 flex-1 truncate text-sm/5 font-medium text-white;
  }

  .tag {
    @apply shrink-0 rounded px-1.5 text-[11px]/[18px] font-semibold text-slate-300 ring-1 ring-white/15;
  }

  /* Two columns of options, as a board's columns are often a handful */
  .options {
    @apply grid grid-cols-2 gap-x-4 gap-y-1.5;
  }

  .option {
    @apply flex min-w-0 cursor-pointer items-center gap-2 rounded-md px-1 py-1 hover:bg-white/5;

    input {
      @apply size-4 shrink-0 accent-pink-500;
    }

    .name {
      @apply font-normal text-slate-200;
    }
  }
</style>
