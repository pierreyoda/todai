import type { UUID } from "node:crypto";
import {
  createMutation,
  type QueryClient,
  useQueryClient,
} from "@tanstack/svelte-query";

import type { Day } from "#core/time";
import { dateToTodaiDate, formatRelativeDay } from "#core/time";
import { tagKeys } from "#domains/tags";
import type { MutationHooks } from "#shared/ipc/mutations.ts";
import { toasts } from "#shared/utils/toasts.svelte.ts";
import { reorderTodos } from "../model/order";
import type { Estimate, Placement, Todo } from "../model/types";
import { invokeTodos } from "./commands";
import {
  cancelTodoListsOf,
  invalidateTodosOf,
  setCachedTodoDescription,
  todoKeys,
} from "./queries";
import { playTodoCompletedSound } from "./sounds";

type NewTodo = Pick<Todo, "day" | "title"> & Partial<Pick<Todo, "tagIds">>;
type TodoUpdate = {
  todo: Pick<Todo, "id" | "day">;
  changes: Partial<
    Pick<Todo, "title" | "day" | "completed" | "urgent" | "tagIds">
  >;
};
type TodoTagging = { todo: Pick<Todo, "id" | "day">; tagIds: readonly UUID[] };

/** Creates a todo at the end of its day. Must be called during component initialization. */
export const createAddTodoMutation = (
  hooks: MutationHooks<NewTodo, Todo> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ day, title, tagIds }: NewTodo) =>
      invokeTodos({ name: "create_todo", args: { day, title, tagIds } }),
    onSuccess: (created, todo) => {
      hooks.onSuccess?.(created, todo);
      // Returned so the mutation stays pending until its day's list is refetched, with it, and the tags' linked todos
      // counts if it has some.
      return Promise.all([
        invalidateTodosOf(queryClient, created.day),
        created.tagIds.length > 0
          ? queryClient.invalidateQueries({ queryKey: tagKeys.all })
          : undefined,
      ]);
    },
    onError: (error, todo) => hooks.onError?.(error, todo),
  }));
};

/**
 * Changes a todo's title, completion, urgency, tags and/or day: moved to another day, it goes at its end. Must be called
 * during component initialization.
 */
export const createUpdateTodoMutation = (
  hooks: MutationHooks<TodoUpdate> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ todo, changes }: TodoUpdate) =>
      invokeTodos({
        name: "update_todo",
        args: { params: { id: todo.id, ...changes } },
      }),
    onSuccess: (_, update) => {
      hooks.onSuccess?.(undefined, update);
      // A todo moved to another day leaves its former one. Returned so the mutation stays pending until the lists are
      // refetched, and the tags' linked todos counts if its tags changed.
      const days = new Set([
        update.todo.day,
        update.changes.day ?? update.todo.day,
      ]);
      return Promise.all([
        ...[...days].map((day) => invalidateTodosOf(queryClient, day)),
        update.changes.tagIds
          ? queryClient.invalidateQueries({ queryKey: tagKeys.all })
          : undefined,
      ]);
    },
    onError: (error, update) => hooks.onError?.(error, update),
  }));
};

/**
 * Sets whether `getTodo()` is completed. Must be called during component initialization.
 *
 * @param hooks Its `onError` lets the caller revert its optimistic state.
 */
export const createToggleTodoMutation = (
  getTodo: () => Todo,
  hooks: MutationHooks<boolean> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (completed: boolean) =>
      invokeTodos({
        name: "toggle_todo",
        args: { id: getTodo().id, completed },
      }),
    // Right away, like the optimistic check, rather than once the lists are refetched
    onMutate: (completed) => {
      if (completed) playTodoCompletedSound();
    },
    onSuccess: (_, completed) => {
      hooks.onSuccess?.(undefined, completed);
      // Returned so the mutation stays pending until the lists are refetched.
      return invalidateTodosOf(queryClient, getTodo().day);
    },
    onError: (error, completed) => hooks.onError?.(error, completed),
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
      invokeTodos({
        name: "set_todo_estimate",
        args: { id: getTodo().id, estimate },
      }),
    // Returned so the mutation stays pending until the lists (and the months' estimate totals) are refetched.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
  }));
};

/** Marks `getTodo()` as urgent, or not. Must be called during component initialization. */
export const createSetTodoUrgentMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (urgent: boolean) =>
      invokeTodos({
        name: "update_todo",
        args: { params: { id: getTodo().id, urgent } },
      }),
    // Returned so the mutation stays pending until the lists are refetched.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
    onError: (_, urgent) => {
      const { title } = getTodo();
      toasts.show(
        `Could not mark “${title}” as ${urgent ? "urgent" : "not urgent"}`,
        { tone: "error" },
      );
    },
  }));
};

/**
 * Sets the description of `getTodo()`, in Markdown, as typed, or removes it if blank. Must be called during component
 * initialization.
 *
 * The saved description replaces the cached one in the lists, rather than refetching them at each autosave.
 *
 * @param optimistic Whether to show the description in the lists right away, then tell if saving it fails and show the
 * saved one back: e.g. for a task checked from the lists. The editor rather tells about failures itself, and keeps the
 * text that couldn't be saved.
 */
export const createSetTodoDescriptionMutation = (
  getTodo: () => Todo,
  { optimistic = false } = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (description: string) =>
      invokeTodos({
        name: "set_todo_description",
        args: { id: getTodo().id, description },
      }),
    // The todo as it was when saving started: the lists to update are its day's, even if it's moved meanwhile
    onMutate: async (description) => {
      const todo = getTodo();
      if (optimistic) {
        // So that a refetch in flight doesn't show the former description back
        await cancelTodoListsOf(queryClient, todo);
        // As the backend stores it: a blank description is none
        setCachedTodoDescription(
          queryClient,
          todo,
          description.trim() === "" ? undefined : description,
        );
      }
      return { todo };
    },
    onSuccess: (saved, _, result) => {
      // Already shown when optimistic: not replaced, as the cache may already show a later change that's being saved
      if (!optimistic)
        setCachedTodoDescription(queryClient, result.todo, saved ?? undefined);
    },
    onError: (_, __, result) => {
      if (!optimistic) return;
      const todo = result?.todo ?? getTodo();
      toasts.show(`Could not save the description of “${todo.title}”`, {
        tone: "error",
      });
      // Returned so the mutation stays pending until the lists are refetched, with the saved description: rather than
      // the one shown before, as other changes may have been saved since.
      return invalidateTodosOf(queryClient, todo.day);
    },
  }));
};

/** Replaces a todo's tags with `tagIds`. Must be called during component initialization. */
export const createSetTodoTagsMutation = (
  hooks: MutationHooks<TodoTagging> = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ todo, tagIds }: TodoTagging) =>
      invokeTodos({ name: "set_todo_tags", args: { todoId: todo.id, tagIds } }),
    onSuccess: (_, tagging) => {
      hooks.onSuccess?.(undefined, tagging);
      // The todo's tags, and the tags' linked todos counts. Returned so the mutation stays pending until they're
      // refetched.
      return Promise.all([
        invalidateTodosOf(queryClient, tagging.todo.day),
        queryClient.invalidateQueries({ queryKey: tagKeys.all }),
      ]);
    },
    onError: (error, tagging) => hooks.onError?.(error, tagging),
  }));
};

/** Moves `todo` to the end of `day`, then refreshes both its days. */
const moveTodo = async (queryClient: QueryClient, todo: Todo, day: Day) => {
  await invokeTodos({
    name: "update_todo",
    args: { params: { id: todo.id, day } },
  });
  await Promise.all(
    [todo.day, day].map((day) => invalidateTodosOf(queryClient, day)),
  );
};

/**
 * Moves `getTodo()` to the end of another day. Must be called during component initialization.
 *
 * @param undoable Whether to tell where it went, offering to move it back (at the end of its former day): for a todo
 * leaving the list it's moved from.
 */
export const createMoveTodoMutation = (
  getTodo: () => Todo,
  { undoable = false } = {},
) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    // Returned so the mutation stays pending until the lists are refetched: the todo leaves its former day for `day`.
    mutationFn: (day: Day) => moveTodo(queryClient, getTodo(), day),
    onSuccess: (_, day) => {
      if (!undoable) return;
      // Kept as it was: the undo outlives the todo's component
      const todo = getTodo();
      const today = dateToTodaiDate(new Date());
      toasts.show(`Moved “${todo.title}” to ${formatRelativeDay(day, today)}`, {
        action: {
          label: "Undo",
          onClick: () =>
            void moveTodo(queryClient, { ...todo, day }, todo.day).catch(() =>
              toasts.show(`Could not move “${todo.title}” back`, {
                tone: "error",
              }),
            ),
        },
      });
    },
  }));
};

/** Moves todos to the end of `getDay()`, in this order, at once. Must be called during component initialization. */
export const createMoveTodosMutation = (getDay: () => Day) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (todos: readonly Todo[]) =>
      invokeTodos({
        name: "move_todos",
        args: { ids: todos.map(({ id }) => id), day: getDay() },
      }),
    // Returned so the mutation stays pending until the lists are refetched: the todos leave their days for `getDay()`.
    onSuccess: (_, todos) => {
      const days = new Set([getDay(), ...todos.map(({ day }) => day)]);
      return Promise.all(
        [...days].map((day) => invalidateTodosOf(queryClient, day)),
      );
    },
  }));
};

/**
 * Moves `todo`, of `getDay()`, right before or after the todo `targetId`: shown so right away, then put back if that
 * fails. Must be called during component initialization.
 */
export const createReorderTodoMutation = (getDay: () => Day) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({
      todo,
      targetId,
      placement,
    }: {
      todo: Todo;
      targetId: Todo["id"];
      placement: Placement;
    }) =>
      invokeTodos({
        name: "reorder_todo",
        args: { id: todo.id, targetId, placement },
      }),
    onMutate: async ({ todo, targetId, placement }) => {
      const queryKey = todoKeys.day(getDay());
      // So that a refetch in flight doesn't show the former order back
      await queryClient.cancelQueries({ queryKey });
      const previous = queryClient.getQueryData<Todo[]>(queryKey);
      if (previous) {
        queryClient.setQueryData(
          queryKey,
          reorderTodos(previous, todo.id, targetId, placement),
        );
      }
      return { previous };
    },
    onError: (_, { todo }, result) => {
      if (result?.previous)
        queryClient.setQueryData(todoKeys.day(getDay()), result.previous);
      toasts.show(`Could not move “${todo.title}”`, { tone: "error" });
    },
    // Returned so the mutation stays pending until the lists, and the calendar's, are refetched in the new order.
    onSettled: () => invalidateTodosOf(queryClient, getDay()),
  }));
};

/** Refreshes what depends on the todos of `day` and on whether they're deleted: tags count their linked todos. */
const invalidateDeletionOf = (queryClient: QueryClient, day: Day) =>
  Promise.all([
    invalidateTodosOf(queryClient, day),
    queryClient.invalidateQueries({ queryKey: tagKeys.all }),
  ]);

/**
 * Restores `todo`, just deleted, at its place: its tags weren't deleted along with it. Tells if it fails, as it's
 * asked from a toast, once the todo's component is gone.
 */
const undoDeletion = async (queryClient: QueryClient, todo: Todo) => {
  try {
    await invokeTodos({
      name: "restore_todo",
      args: { id: todo.id, tagIds: [] },
    });
  } catch {
    toasts.show(`Could not restore “${todo.title}”`, { tone: "error" });
    return;
  }
  await invalidateDeletionOf(queryClient, todo.day);
};

/** Deletes `getTodo()`, offering to undo it. Must be called during component initialization. */
export const createDeleteTodoMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () =>
      invokeTodos({
        name: "delete_todo",
        args: { id: getTodo().id },
      }),
    onSuccess: () => {
      // Kept as it is now: the undo outlives the todo's component
      const todo = getTodo();
      toasts.show(`Deleted “${todo.title}”`, {
        action: {
          label: "Undo",
          onClick: () => void undoDeletion(queryClient, todo),
        },
      });
      // Returned so the mutation stays pending until the lists are refetched.
      return invalidateDeletionOf(queryClient, todo.day);
    },
  }));
};
