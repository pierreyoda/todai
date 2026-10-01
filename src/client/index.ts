import { invoke } from "@tauri-apps/api/core";
import type { Day, Todo } from "./types";

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
};

export type ClientCommandName = keyof ClientInvocationCommand;

export const invokeClient = async <N extends ClientCommandName, R = ClientInvocationCommand[N]["returns"]>(c: { name: N; args: ClientInvocationCommand[N]["args"] }): Promise<R> =>
  invoke<R>(c.name, c.args).catch((e) => {
    console.error(`Error invoking command ${c.name}:`, e);
    throw e;
  });
