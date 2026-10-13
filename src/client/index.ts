import type { Day, Month } from "#core/time";
import type {
  Estimate,
  LinkPreview,
  Note,
  ParsedLink,
  Placement,
  RefreshOutcome,
  Todo,
  TodoLink,
  TodoMonth,
} from "./types";
import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";

type ClientInvocationCommand = {
  list_todos: {
    args: {
      day: Day,
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
  create_todo: {
    args: {
      day: Day;
      title: string;
      description?: string;
      estimate?: Estimate;
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
  /** Fields left out are unchanged; moving a todo to another day puts it at the end of that day. */
  update_todo: {
    args: {
      params: {
        id: UUID;
        title?: string;
        day?: Day;
        completed?: boolean;
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
  /**
   * Recognizes what `url` points to, without adding it anywhere.
   *
   * Fails if it's not an http(s) URL (the scheme can be left out: `github.com/owner/repo/pull/1`).
   */
  parse_link: {
    args: {
      url: string;
    };
    returns: ParsedLink;
  };
  /**
   * Adds the links `urls`, each normalized as by `parse_link`, in this order after the other links of a non-deleted
   * todo.
   *
   * Fails without adding any if one is not an http(s) URL, if the todo already has one, or if two are the same.
   */
  add_todo_links: {
    args: {
      todoId: UUID;
      urls: readonly string[];
    };
    returns: TodoLink[];
  };
  /**
   * Creates a todo at the end of `day` with the link `url`, at once: titled `title` (e.g. its pull request's, from
   * `preview_link`), or else as the link is shown (e.g. "todai#58").
   */
  create_todo_from_link: {
    args: {
      day: Day;
      url: string;
      title?: string;
    };
    returns: Todo;
  };
  /**
   * Recognizes what `url` points to, as `parse_link`, then, for a GitHub pull request or issue, asks GitHub about it
   * through the active workspace's connection, if any. Nothing is stored.
   */
  preview_link: {
    args: {
      url: string;
    };
    returns: LinkPreview;
  };
  /** Removes a link from its non-deleted todo. */
  remove_todo_link: {
    args: {
      id: UUID;
    };
    returns: never;
  };
  /**
   * Refreshes the status of the pull requests and issues linked to the non-deleted todos of `day`, if fetched long
   * enough ago (5 minutes for open ones, a day for the others, or whenever if `force`), through the active workspace's
   * GitHub connection. Does nothing without one.
   */
  refresh_links: {
    args: {
      day: Day;
      force?: boolean;
    };
    returns: RefreshOutcome;
  };
  /** Refreshes the status of a GitHub link's pull request or issue, however recently fetched, as `refresh_links`. */
  refresh_link: {
    args: {
      id: UUID;
    };
    returns: RefreshOutcome;
  };
  /** `null` if the day has no note. */
  get_note: {
    args: {
      day: Day;
    };
    returns: Note | null;
  };
  /** The notes from `start` to `end` (inclusive), by day. */
  list_notes_between: {
    args: {
      start: Day;
      end: Day;
    };
    returns: Note[];
  };
  /** Months having notes, most recent first. */
  list_note_months: {
    args?: never;
    returns: Month[];
  };
  /**
   * Saves `content` as the note of `day`, as typed (not trimmed). A blank `content` deletes the note instead, returning
   * `null`. Saving the same content again leaves its `updatedAt` unchanged.
   */
  save_note: {
    args: {
      day: Day;
      content: string;
    };
    returns: Note | null;
  };
};

export const invokeClient = createInvoker<ClientInvocationCommand>("command");
