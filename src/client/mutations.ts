import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import { invokeClient } from ".";
import { invokeApiClient, type Workspace } from "./app";
import type { Day, Estimate, Todo, TodoLink } from "./types";
import { ghKeys, invalidateTodosOf, noteKeys, tagKeys, workspaceKeys } from "./queries";
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
 * Adds a link to `getTodo()`, from a URL as typed or pasted: normalized by the backend. Must be called during component
 * initialization.
 */
export const createAddTodoLinkMutation = (getTodo: () => Todo) => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: (url: string) =>
      invokeClient({
        name: "add_todo_link",
        args: { todoId: getTodo().id, url },
      }),
    // Returned so the mutation stays pending until the lists are refetched, with the link.
    onSuccess: () => invalidateTodosOf(queryClient, getTodo().day),
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
    // Returned so the mutation stays pending until the workspaces are refetched, with the connection.
    onSuccess: () => queryClient.invalidateQueries({ queryKey: workspaceKeys.all }),
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
