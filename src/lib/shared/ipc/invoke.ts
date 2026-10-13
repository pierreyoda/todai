import { type InvokeArgs, invoke } from "@tauri-apps/api/core";

import { parseResponse } from "./parseResponse";

/** Backend commands, by name: what each takes (`args`, if anything) and returns. */
export type Commands = Record<string, { args?: InvokeArgs; returns: unknown }>;

/** `args` can be left out for the commands taking none. */
export type Invocation<C extends Commands, N extends keyof C> = {
  name: N;
} & (undefined extends C[N]["args"]
  ? { args?: C[N]["args"] }
  : { args: C[N]["args"] });

/**
 * Invokes the backend's commands `C`, resolving to their response as typed in the frontend (see `parseResponse`).
 * Failures are logged as `kind`'s, e.g. "app command".
 */
export const createInvoker =
  <C extends Commands>(kind: string) =>
  async <N extends keyof C & string, R = C[N]["returns"]>(
    c: Invocation<C, N>,
  ): Promise<R> =>
    invoke(c.name, c.args)
      .then(parseResponse<R>)
      .catch((e) => {
        console.error(`Error invoking ${kind} ${c.name}:`, e);
        throw e;
      });
