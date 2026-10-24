import type { UUID } from "node:crypto";

import type { Day } from "#core/time";
import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Estimate, Placement, Todo, TodoMonth } from "../model/types";

type TodoCommands = {
  list_todos: {
    args: {
      day: Day;
    };
    returns: Todo[];
  };
  /** Todos from `start` to `end` (inclusive), by day then in display order. */
  list_todos_between: {
    args: {
      start: Day;
      end: Day;
    };
    returns: Todo[];
  };
  /** Months having todos, most recent first. */
  list_todo_months: {
    args?: never;
    returns: TodoMonth[];
  };
  /** Creates nothing if one of `tagIds` isn't a non-deleted tag. */
  create_todo: {
    args: {
      day: Day;
      title: string;
      description?: string;
      estimate?: Estimate;
      tagIds?: readonly UUID[];
    };
    returns: Todo;
  };
  toggle_todo: {
    args: {
      id: UUID;
      completed: boolean;
    };
    returns: never;
  };
  delete_todo: {
    args: {
      id: UUID;
    };
    returns: never;
  };
  /**
   * Restores a deleted todo to its day, at its former position, along with `tagIds`: some of its deleted tags, which
   * then come back on their other todos too.
   *
   * Fails without restoring anything if the todo isn't deleted, or if any of `tagIds` isn't one of its deleted tags.
   */
  restore_todo: {
    args: {
      id: UUID;
      tagIds: readonly UUID[];
    };
    returns: never;
  };
  /**
   * Fields left out are unchanged; moving a todo to another day puts it at the end of that day. `started` starts it,
   * keeping when it was first started if it already is, or clears when it was started. `tagIds` replaces its tags, as
   * `set_todo_tags`: changes nothing if one of them isn't a non-deleted tag.
   */
  update_todo: {
    args: {
      params: {
        id: UUID;
        title?: string;
        day?: Day;
        completed?: boolean;
        urgent?: boolean;
        started?: boolean;
        tagIds?: readonly UUID[];
      };
    };
    returns: never;
  };
  /**
   * Moves todos to the end of `day`, in this order, as `update_todo` would one by one: those already on `day` stay in
   * place. Fails without moving any if one of them is deleted or unknown.
   */
  move_todos: {
    args: {
      ids: readonly UUID[];
      day: Day;
    };
    returns: never;
  };
  /**
   * Moves a todo right before or after `targetId`, a todo of the same day, among all the todos of that day: e.g. dropped
   * there in a list showing only some of them. Moving it next to itself changes nothing.
   *
   * Fails if either is deleted or unknown, or if they're not on the same day.
   */
  reorder_todo: {
    args: {
      id: UUID;
      targetId: UUID;
      placement: Placement;
    };
    returns: never;
  };
  /** Replaces the todo's estimate, whatever its unit, or removes it if `null`. */
  set_todo_estimate: {
    args: {
      id: UUID;
      estimate: Estimate | null;
    };
    returns: never;
  };
  /**
   * Sets the todo's description, in Markdown, as typed (not trimmed), or removes it if blank. Returns it as stored:
   * `null` once removed.
   */
  set_todo_description: {
    args: {
      id: UUID;
      description: string;
    };
    returns: string | null;
  };
  /** Replaces the todo's tags with `tagIds`. */
  set_todo_tags: {
    args: {
      todoId: UUID;
      tagIds: readonly UUID[];
    };
    returns: never;
  };
};

export const invokeTodos = createInvoker<TodoCommands>("command");
