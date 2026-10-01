import type { PageLoad } from "./$types"; ``

import { invokeClient } from "../client";
import { dateToTodaiDate } from "../utils/dates";

export const load: PageLoad = async () => ({
  todos: await invokeClient({
    name: "list_todos",
    args: { day: dateToTodaiDate(new Date()) },
  }),
});
