import { invoke } from "@tauri-apps/api/core";
import type {
  Day,
  Estimate,
  LinkPreview,
  Note,
  ParsedLink,
  RefreshOutcome,
  Tag,
  Todo,
  TodoLink,
  TodoMonth,
} from "./types";
import type { UUID } from "node:crypto";

import { parseResponse } from "../utils/client";

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
  /** Replaces the todo's estimate, whatever its unit, or removes it if `null`. */
  set_todo_estimate: {
    args: {
      id: UUID;
      estimate: Estimate | null;
    };
    returns: never;
  };
  list_tags: {
    args?: never;
    returns: Tag[];
  };
  create_tag: {
    args: {
      name: string;
      /** Format: #RRGGBB. */
      color: string;
    };
    returns: Tag;
  };
  update_tag: {
    args: {
      id: UUID;
      name?: string;
      /** Format: #RRGGBB. */
      color?: string;
    };
    returns: Tag;
  };
  delete_tag: {
    args: {
      id: UUID;
    };
    returns: never;
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

export type ClientCommandName = keyof ClientInvocationCommand;

/** `args` can be left out for the commands taking none. */
export type Invocation<C extends Record<string, { args?: unknown }>, N extends keyof C> = { name: N } & (undefined extends C[N]["args"]
  ? { args?: C[N]["args"] }
  : { args: C[N]["args"] });

export const invokeClient = async <N extends ClientCommandName, R = ClientInvocationCommand[N]["returns"]>(c: Invocation<ClientInvocationCommand, N>): Promise<R> =>
  invoke(c.name, c.args).then(parseResponse<R>).catch((e) => {
    console.error(`Error invoking command ${c.name}:`, e);
    throw e;
  });
