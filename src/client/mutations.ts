import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import type { Day, Estimate, Todo } from "./types";
import { invalidateTodosOf, noteKeys, tagKeys } from "./queries";
import { playTodoCompletedSound } from "../utils/sounds";

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
    // Right away, like the optimistic check, rather than once the lists are refetched
    onMutate: (completed) => {
      if (completed) playTodoCompletedSound();
    },
    // Returned so the mutation stays pending until the lists are refetched.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
    onError,
  }));
};

/**
 * Sets the estimate of `getTodo()`, replacing any previous one whatever its unit, or removes it if `null`.
 * Must be called during component initialization.
 */
export const createSetTodoEstimateMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (estimate: Estimate | null) =>
      invokeClient({
        name: "set_todo_estimate",
        args: { id: getTodo().id, estimate },
      }),
    // Returned so the mutation stays pending until the lists (and the months' estimate totals) are refetched.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
  }));
};

/** Moves `getTodo()` to the end of another day. Must be called during component initialization. */
export const createMoveTodoMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (day: Day) =>
      invokeClient({
        name: "update_todo",
        args: { params: { id: getTodo().id, day } },
      }),
    // Returned so the mutation stays pending until the lists are refetched: the todo leaves its former day for `day`.
    onSuccess: (_, day) =>
      Promise.all(
        [getTodo().day, day].map((day) => invalidateTodosOf(queryClient, day)),
      ),
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

/**
 * Saves `content` as the note of `day`, as typed, or deletes it if blank. Must be called during component
 * initialization.
 *
 * The day is passed with each save, rather than read once saved: a slow save still updates the note it was made for.
 */
export const createSaveNoteMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ day, content }: { day: Day; content: string }) =>
      invokeClient({ name: "save_note", args: { day, content } }),
    // The saved note (or `null`, once deleted) replaces the cached one, rather than being refetched.
    onSuccess: (note, { day }) => queryClient.setQueryData(noteKeys.day(day), note),
  }));
};
