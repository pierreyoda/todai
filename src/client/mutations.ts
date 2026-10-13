import { createMutation, useQueryClient, type QueryClient } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import { invokeApiClient, type Workspace } from "./app";
import type { Day, Estimate, Placement, Todo, TodoLink } from "./types";
import {
  cancelTodoListsOf,
  ghKeys,
  invalidateActiveWorkspace,
  invalidateTodosOf,
  linkStatusKeys,
  noteKeys,
  refreshGitHubAttention,
  refreshLinksOf,
  setCachedTodoDescription,
  todoKeys,
  workspaceKeys,
} from "./queries";
import { tagKeys } from "#domains/tags";
import { playTodoCompletedSound } from "../utils/sounds";
import { toasts } from "#shared/utils/toasts.svelte.ts";
import { dateToTodaiDate, formatRelativeDay } from "#core/time";
import { canSwitchWorkspace } from "../utils/workspaceSwitch";
import { reorderTodos } from "../utils/todoOrder";

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
export const createSetTodoDescriptionMutation = (getTodo: () => Todo, { optimistic = false } = {}) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (description: string) =>
      invokeClient({
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
        setCachedTodoDescription(queryClient, todo, description.trim() === "" ? undefined : description);
      }
      return { todo };
    },
    onSuccess: (saved, _, result) => {
      // Already shown when optimistic: not replaced, as the cache may already show a later change that's being saved
      if (!optimistic) setCachedTodoDescription(queryClient, result.todo, saved ?? undefined);
    },
    onError: (_, __, result) => {
      if (!optimistic) return;
      const todo = result?.todo ?? getTodo();
      toasts.show(`Could not save the description of “${todo.title}”`, { tone: "error" });
      // Returned so the mutation stays pending until the lists are refetched, with the saved description: rather than
      // the one shown before, as other changes may have been saved since.
      return invalidateTodosOf(queryClient, todo.day);
    },
  }));
};

/** Moves `todo` to the end of `day`, then refreshes both its days. */
const moveTodo = async (queryClient: QueryClient, todo: Todo, day: Day) => {
  await invokeClient({ name: "update_todo", args: { params: { id: todo.id, day } } });
  await Promise.all([todo.day, day].map((day) => invalidateTodosOf(queryClient, day)));
};

/**
 * Moves `getTodo()` to the end of another day. Must be called during component initialization.
 *
 * @param undoable Whether to tell where it went, offering to move it back (at the end of its former day): for a todo
 * leaving the list it's moved from.
 */
export const createMoveTodoMutation = (getTodo: () => Todo, { undoable = false } = {}) => {
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
              toasts.show(`Could not move “${todo.title}” back`, { tone: "error" }),
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
      invokeClient({
        name: "move_todos",
        args: { ids: todos.map(({ id }) => id), day: getDay() },
      }),
    // Returned so the mutation stays pending until the lists are refetched: the todos leave their days for `getDay()`.
    onSuccess: (_, todos) => {
      const days = new Set([getDay(), ...todos.map(({ day }) => day)]);
      return Promise.all([...days].map((day) => invalidateTodosOf(queryClient, day)));
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
    mutationFn: ({ todo, targetId, placement }: { todo: Todo; targetId: Todo["id"]; placement: Placement }) =>
      invokeClient({ name: "reorder_todo", args: { id: todo.id, targetId, placement } }),
    onMutate: async ({ todo, targetId, placement }) => {
      const queryKey = todoKeys.day(getDay());
      // So that a refetch in flight doesn't show the former order back
      await queryClient.cancelQueries({ queryKey });
      const previous = queryClient.getQueryData<Todo[]>(queryKey);
      if (previous) {
        queryClient.setQueryData(queryKey, reorderTodos(previous, todo.id, targetId, placement));
      }
      return { previous };
    },
    onError: (_, { todo }, result) => {
      if (result?.previous) queryClient.setQueryData(todoKeys.day(getDay()), result.previous);
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
    await invokeClient({ name: "restore_todo", args: { id: todo.id, tagIds: [] } });
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
      invokeClient({
        name: "delete_todo",
        args: { id: getTodo().id },
      }),
    onSuccess: () => {
      // Kept as it is now: the undo outlives the todo's component
      const todo = getTodo();
      toasts.show(`Deleted “${todo.title}”`, {
        action: { label: "Undo", onClick: () => void undoDeletion(queryClient, todo) },
      });
      // Returned so the mutation stays pending until the lists are refetched.
      return invalidateDeletionOf(queryClient, todo.day);
    },
  }));
};

/**
 * Adds links to `getTodo()`, in this order, from URLs as typed or pasted: normalized by the backend. Must be called
 * during component initialization.
 */
export const createAddTodoLinksMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (urls: readonly string[]) =>
      invokeClient({
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
      invokeClient({ name: "create_todo_from_link", args: { day: getDay(), url, title } }),
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
      invokeClient({
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
    mutationFn: (link: TodoLink) => invokeClient({ name: "refresh_link", args: { id: link.id } }),
    // Returned so the mutation stays pending until the lists are refetched, with the status.
    onSuccess: (outcome) => {
      refreshGitHubAttention(queryClient);
      if (outcome.refreshed > 0) return invalidateTodosOf(queryClient, getTodo().day);
    },
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
    // The saved note (or `null`, once deleted) replaces the cached one, rather than being refetched. The calendar's
    // notes are only marked stale: refetched once shown, rather than at each autosave.
    onSuccess: (note, { day }) => {
      queryClient.setQueryData(noteKeys.day(day), note);
      void queryClient.invalidateQueries({ queryKey: noteKeys.months(), refetchType: "none" });
    },
  }));
};

/**
 * Connects a workspace to GitHub through a GitHub CLI account, instead of any other one. Must be called during
 * component initialization.
 */
export const createConnectGitHubMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ workspace, login }: { workspace: Workspace; login: string }) =>
      invokeApiClient({
        name: "connect_github",
        args: { workspaceId: workspace.id, login },
      }),
    onSuccess: () => {
      // The links' statuses, through the new connection, without waiting for GitHub
      queryClient.invalidateQueries({ queryKey: linkStatusKeys.all });
      // Returned so the mutation stays pending until the workspaces are refetched, with the connection.
      return queryClient.invalidateQueries({ queryKey: workspaceKeys.all });
    },
  }));
};

/** Disconnects a workspace from GitHub. Must be called during component initialization. */
export const createDisconnectGitHubMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (workspace: Workspace) =>
      invokeApiClient({
        name: "disconnect_github",
        args: { workspaceId: workspace.id },
      }),
    // Returned so the mutation stays pending until the workspaces are refetched, without the connection.
    onSuccess: () => queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
  }));
};

/**
 * Looks for the GitHub CLI again rather than where it was found, e.g. once installed or signed in to. Must be called
 * during component initialization.
 */
export const createRefindGhMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: () => invokeApiClient({ name: "get_gh_status", args: { refind: true } }),
    // Replaces the cached status, rather than refetching it without looking for `gh` again.
    onSuccess: (status) => queryClient.setQueryData(ghKeys.status(), status),
  }));
};

/**
 * Switches to another workspace, once what's being edited in the current one is saved (see `beforeWorkspaceSwitch`):
 * resolves to whether it did. Must be called during component initialization.
 */
export const createSwitchWorkspaceMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: async ({ id }: Workspace) => {
      if (!(await canSwitchWorkspace())) return false;
      await invokeApiClient({ name: "switch_to_workspace", args: { id } });
      return true;
    },
    // Returned so the mutation stays pending until everything is refetched from the other workspace's database.
    onSuccess: (switched) => {
      if (!switched) return;
      return Promise.all([
        // Dropped rather than only refetched: the note's editor is kept while its note is shown, and must start over
        queryClient.resetQueries({ queryKey: noteKeys.all }),
        invalidateActiveWorkspace(queryClient),
      ]);
    },
  }));
};
