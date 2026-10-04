import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import type { Todo } from "./types";
import { invalidateTodosOf, tagKeys } from "./queries";

// TODO: migrate other mutations to this file

/**
 * Sets whether `getTodo()` is completed. Must be called during component initialization.
 *
 * @param onError Lets the caller revert its optimistic state.
 */
export const createToggleTodoMutation = (getTodo: () => Todo, onError?: () => void) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (completed: boolean) =>
      invokeClient({
        name: "toggle_todo",
        args: { id: getTodo().id, completed },
      }),
    // Returned so the mutation stays pending until the lists are refetched.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
    onError,
  }));
};

/** Deletes `getTodo()`. Must be called during component initialization. */
export const createDeleteTodoMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () =>
      invokeClient({
        name: "delete_todo",
        args: { id: getTodo().id },
      }),
    // Returned so the mutation stays pending until the lists are refetched; tags count their linked todos.
    onSuccess: () =>
      Promise.all([
        invalidateTodosOf(queryClient, getTodo().day),
        queryClient.invalidateQueries({ queryKey: tagKeys.all }),
      ]),
  }));
};
