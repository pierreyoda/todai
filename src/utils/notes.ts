import type { Day } from "../client/types";

/** Where the note of `day` is read and written: Today's sub-page for today's, the calendar's for the others'. */
export const notePath = (day: Day, today: Day): string =>
  day === today ? "/today/notes" : `/calendar/notes/${day}`;
