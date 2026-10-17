import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import type { Day } from "#core/time";
import { refreshIntegrationsAttention } from "#domains/workspaces";
import type { MutationHooks } from "#shared/ipc/mutations.ts";
import { invalidateTodosOf, todoKeys } from "../../api/queries";
import type { Todo } from "../../model/types";
import type { NotionDoneRule, TodoLink } from "../model/types";
import { invokeLinks, invokeNotionRules } from "./commands";
import { linkStatusKeys, notionDoneRuleKeys, refreshLinksOf } from "./queries";

/**
 * Adds links to `getTodo()`, in this order, from URLs as typed or pasted: normalized by the backend. Must be called
 * during component initialization.
 */
export const createAddTodoLinksMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (urls: readonly string[]) =>
      invokeLinks({
        name: "add_todo_links",
        args: { todoId: getTodo().id, urls },
      }),
    onSuccess: () => {
      // Their status, if they have one, even if not refreshed on their own; without waiting for their provider
      refreshLinksOf(queryClient, getTodo().day);
      // Returned so the mutation stays pending until the lists are refetched, with the links.
      return invalidateTodosOf(queryClient, getTodo().day);
    },
  }));
};

/**
 * Creates a todo at the end of `getDay()` with a link, titled after its pull request or issue if known. Must be called
 * during component initialization.
 */
export const createAddTodoFromLinkMutation = (getDay: () => Day) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ url, title }: { url: string; title?: string }) =>
      invokeLinks({
        name: "create_todo_from_link",
        args: { day: getDay(), url, title },
      }),
    onSuccess: () => {
      refreshLinksOf(queryClient, getDay());
      // Returned so the mutation stays pending until the list is refetched, with the todo.
      return invalidateTodosOf(queryClient, getDay());
    },
  }));
};

/** Removes one of the links of `getTodo()`. Must be called during component initialization. */
export const createRemoveTodoLinkMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (link: TodoLink) =>
      invokeLinks({
        name: "remove_todo_link",
        args: { id: link.id },
      }),
    // Returned so the mutation stays pending until the lists are refetched, without the link.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
  }));
};

/**
 * Fetches again the status of a link of `getTodo()`, however recently fetched. Must be called during component
 * initialization.
 *
 * Succeeds with the problem its provider couldn't be asked because of, if any: the link then keeps its last known
 * status.
 */
export const createRefreshTodoLinkMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (link: TodoLink) =>
      invokeLinks({ name: "refresh_link", args: { id: link.id } }),
    // Returned so the mutation stays pending until the lists are refetched, with the status.
    onSuccess: (outcome) => {
      refreshIntegrationsAttention(queryClient);
      if (outcome.refreshed > 0)
        return invalidateTodosOf(queryClient, getTodo().day);
    },
  }));
};

/** Refreshes what a done rule changes: the rules, and how every todo's Notion pages are shown. */
const afterRuleChange = (queryClient: ReturnType<typeof useQueryClient>) =>
  Promise.all([
    queryClient.invalidateQueries({ queryKey: notionDoneRuleKeys.list() }),
    queryClient.invalidateQueries({ queryKey: todoKeys.all }),
    queryClient.invalidateQueries({ queryKey: linkStatusKeys.all }),
  ]);

/**
 * Sets the done rule of a Notion database, instead of any other one: every todo linked to one of its pages then shows
 * it as the rule tells. Must be called during component initialization.
 */
export const createSetNotionDoneRuleMutation = (
  hooks: MutationHooks<NotionDoneRule, NotionDoneRule> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (rule: NotionDoneRule) =>
      invokeNotionRules({ name: "set_notion_done_rule", args: { rule } }),
    onSuccess: (saved, rule) => {
      hooks.onSuccess?.(saved, rule);
      return afterRuleChange(queryClient);
    },
    onError: (error, rule) => hooks.onError?.(error, rule),
  }));
};

/**
 * Deletes the done rule of a Notion database, by its data source's ID: its pages are then done once their Status is in
 * its Complete group again. Must be called during component initialization.
 */
export const createDeleteNotionDoneRuleMutation = (
  hooks: MutationHooks<string, boolean> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (dataSourceId: string) =>
      invokeNotionRules({
        name: "delete_notion_done_rule",
        args: { dataSourceId },
      }),
    onSuccess: (deleted, dataSourceId) => {
      hooks.onSuccess?.(deleted, dataSourceId);
      return afterRuleChange(queryClient);
    },
    onError: (error, dataSourceId) => hooks.onError?.(error, dataSourceId),
  }));
};
