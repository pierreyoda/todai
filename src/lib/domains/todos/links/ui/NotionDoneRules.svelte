<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import Button from "#shared/ui/Button.svelte";
import ErrorBanner from "#shared/ui/ErrorBanner.svelte";
import { createDeleteNotionDoneRuleMutation } from "../api/mutations";
import { notionDoneRulesQueryOptions } from "../api/queries";
import { DEFAULT_RULE, describeRule } from "../model/rules";
import type { NotionDoneRule } from "../model/types";
import NotionDoneRuleModal from "./NotionDoneRuleModal.svelte";

/** The done rules of the active workspace's Notion databases, to edit or reset: in the integrations settings. */
const rules = createQuery(() => notionDoneRulesQueryOptions);
const reset = createDeleteNotionDoneRuleMutation();

let editing = $state<NotionDoneRule | null>(null);
let showModal = $state(false);
const edit = (rule: NotionDoneRule) => {
  editing = rule;
  showModal = true;
};
</script>

<section class="rules" aria-labelledby="notion-done-rules-title">
  <h4 id="notion-done-rules-title" class="title">When pages are done</h4>
  {#if rules.isError}
    <ErrorBanner title="Could not list the done rules" error={rules.error} />
  {:else if rules.data && rules.data.length > 0}
    <ul class="list">
      {#each rules.data as rule (rule.dataSourceId)}
        <li class="rule">
          <span class="text">
            <span class="database">{rule.databaseTitle}</span>: done when
            {describeRule(rule)}
          </span>
          <Button style="outline" size="xs" onclick={() => edit(rule)}>
            Edit…
          </Button>
          <Button
            style="plain"
            size="xs"
            disabled={reset.isPending}
            onclick={() => reset.mutate(rule.dataSourceId)}
          >
            Reset
          </Button>
        </li>
      {/each}
    </ul>
    <p class="hint">
      Other databases' pages are done once their Status is in its Complete
      group.
    </p>
  {:else if rules.data}
    <p class="hint">
      A page is done once {DEFAULT_RULE}. To use another column of a database
      (e.g. a Stage select), click one of its pages' links, then Change… next to
      “Done when”.
    </p>
  {/if}
  {#if reset.isError}
    <ErrorBanner
      title="Could not reset the rule"
      error={reset.error}
      onDismiss={() => reset.reset()}
    />
  {/if}
</section>
{#if editing}
  <NotionDoneRuleModal
    bind:show={showModal}
    dataSourceId={editing.dataSourceId}
    databaseTitle={editing.databaseTitle}
  />
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  .rules {
    @apply flex flex-col gap-2 border-t border-white/6 pt-3;
  }

  .title {
    @apply text-[13px]/5 font-medium text-white;
  }

  .list {
    @apply flex flex-col gap-1.5;
  }

  .rule {
    @apply flex items-center gap-2;
  }

  .text {
    @apply min-w-0 flex-1 text-[13px]/5 text-slate-300;
  }

  .database {
    @apply font-medium text-white;
  }

  .hint {
    @apply text-xs/5 text-slate-400;
  }
</style>
