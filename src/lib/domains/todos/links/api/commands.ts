import type { UUID } from "node:crypto";

import type { Day } from "#core/time";
import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Todo } from "../../model/types";
import type {
  LinkPreview,
  NotionColumns,
  NotionDoneRule,
  ParsedLink,
  RefreshOutcome,
  TodoLink,
} from "../model/types";

type LinkCommands = {
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
   * Recognizes what `url` points to, as `parse_link`, then, for an item fetched from a provider (e.g. a GitHub pull
   * request or issue), asks it about it through the active workspace's connection, if any. Nothing is stored.
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
   * Refreshes the status of the items (e.g. pull requests and issues) linked to the non-deleted todos of `day`, if
   * fetched long enough ago (5 minutes for open ones, a day for the others, or whenever if `force`), through the active
   * workspace's connection to their provider. Does nothing for a provider it isn't connected to.
   */
  refresh_links: {
    args: {
      day: Day;
      force?: boolean;
    };
    returns: RefreshOutcome;
  };
  /** Refreshes the status of a link's item (e.g. a pull request), however recently fetched, as `refresh_links`. */
  refresh_link: {
    args: {
      id: UUID;
    };
    returns: RefreshOutcome;
  };
};

export const invokeLinks = createInvoker<LinkCommands>("command");

type NotionRuleCommands = {
  /** The done rules of the active workspace's Notion databases, by their title. */
  list_notion_done_rules: {
    returns: NotionDoneRule[];
  };
  /** Instead of any other one for its database. Fails without a value meaning done, unless it's a checkbox. */
  set_notion_done_rule: {
    args: {
      rule: NotionDoneRule;
    };
    returns: NotionDoneRule;
  };
  /** Whether there was one: the database's pages are then done once their Status is in its Complete group again. */
  delete_notion_done_rule: {
    args: {
      dataSourceId: string;
    };
    returns: boolean;
  };
  /**
   * Asks Notion, now, for a database's columns that can tell its pages are done. Fails if the active workspace isn't
   * connected to Notion, Notion couldn't be asked, or its token can't read the database.
   */
  get_notion_columns: {
    args: {
      dataSourceId: string;
    };
    returns: NotionColumns;
  };
};

export const invokeNotionRules = createInvoker<NotionRuleCommands>("command");
