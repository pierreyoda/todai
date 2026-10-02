import { invoke } from "@tauri-apps/api/core";
import type { Day, Tag, Todo } from "./types";
import type { UUID } from "node:crypto";

type ClientInvocationCommand = {
  list_todos: {
    args: {
      day: Day,
    };
    returns: Todo[];
  };
  create_todo: {
    args: {
      day: Day;
      title: string;
      description?: string;
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
  list_tags: {
    args: Record<string, never>;
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
};

export type ClientCommandName = keyof ClientInvocationCommand;

export const invokeClient = async <N extends ClientCommandName, R = ClientInvocationCommand[N]["returns"]>(c: { name: N; args: ClientInvocationCommand[N]["args"] }): Promise<R> =>
  invoke<R>(c.name, c.args).catch((e) => {
    console.error(`Error invoking command ${c.name}:`, e);
    throw e;
  });
