import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import type { Day } from "#core/time";
import { toasts } from "#shared/utils/toasts.svelte.ts";
import { invokeTodos } from "../../api/commands";
import { invalidateTodosOf, todoKeys } from "../../api/queries";
import { playTodoCompletedSound } from "../../api/sounds";
import type { Todo } from "../../model/types";
import { applyMove, type BoardMove } from "../model/board";

type TodoBoardMove = { todo: Todo; move: BoardMove };

/**
 * Moves `todo`, of `getDay()`, on the board (see `planDrop`): to another column, then next to another todo, at once.
 * Shown so right away, then put back if that fails. Must be called during component initialization.
 */
export const createMoveOnBoardMutation = (getDay: () => Day) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: async ({ todo, move: { changes, next } }: TodoBoardMove) => {
      if (changes) {
        await invokeTodos({
          name: "update_todo",
          args: { params: { id: todo.id, ...changes } },
        });
      }
      if (next) {
        await invokeTodos({
          name: "reorder_todo",
          args: { id: todo.id, ...next },
        });
      }
    },
    onMutate: async ({ todo, move }) => {
      // Right away, as the list's checkbox does
      if (move.changes?.completed && !todo.completed) playTodoCompletedSound();
      const queryKey = todoKeys.day(getDay());
      // So that a refetch in flight doesn't show it back where it was
      await queryClient.cancelQueries({ queryKey });
      const previous = queryClient.getQueryData<Todo[]>(queryKey);
      if (previous) {
        queryClient.setQueryData(
          queryKey,
          applyMove(previous, todo.id, move, new Date()),
        );
      }
      return { previous };
    },
    onError: (_, { todo }, result) => {
      if (result?.previous)
        queryClient.setQueryData(todoKeys.day(getDay()), result.previous);
      toasts.show(`Could not move “${todo.title}”`, { tone: "error" });
    },
    // Returned so the mutation stays pending until the lists, and the calendar's, are refetched as stored: e.g. moved to
    // another column even though it couldn't be reordered.
    onSettled: () => invalidateTodosOf(queryClient, getDay()),
  }));
};
