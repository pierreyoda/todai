import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import type { Day } from "#core/time";
import { refreshGitHubAttention } from "#domains/workspaces";
import { invalidateTodosOf } from "../../api/queries";
import type { Todo } from "../../model/types";
import type { TodoLink } from "../model/types";
import { invokeLinks } from "./commands";
import { refreshLinksOf } from "./queries";

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
      // Their status, if they're GitHub ones, even if not refreshed on their own; without waiting for GitHub
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
 * Fetches again the status of a GitHub link of `getTodo()`, however recently fetched. Must be called during component
 * initialization.
 *
 * Succeeds with the problem GitHub couldn't be asked because of, if any: the link then keeps its last known status.
 */
export const createRefreshTodoLinkMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (link: TodoLink) =>
      invokeLinks({ name: "refresh_link", args: { id: link.id } }),
    // Returned so the mutation stays pending until the lists are refetched, with the status.
    onSuccess: (outcome) => {
      refreshGitHubAttention(queryClient);
      if (outcome.refreshed > 0)
        return invalidateTodosOf(queryClient, getTodo().day);
    },
  }));
};
