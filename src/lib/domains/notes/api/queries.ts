import { queryOptions } from "@tanstack/svelte-query";

import type { Day, Month } from "#core/time";
import { monthBounds } from "#core/time";
import { afterWorkspaceSwitch } from "#domains/workspaces";
import { queryClient, workspaceDataKey } from "#shared/ipc/queryClient.ts";
import { invokeNotes } from "./commands";

/**
 * Days are `YYYY-MM-DD`, so they never collide with `"months"`. The months' notes are nested in the list of months, so
 * that refreshing it after saving a note also refreshes them.
 */
export const noteKeys = {
  all: [...workspaceDataKey, "notes"] as const,
  day: (day: Day) => [...noteKeys.all, day] as const,
  months: () => [...noteKeys.all, "months"] as const,
  month: (month: Month) => [...noteKeys.months(), month] as const,
};

// Dropped rather than only refetched when switching to another workspace: the note's editor is kept while its note is
// shown, and must start over
afterWorkspaceSwitch(() =>
  queryClient.resetQueries({ queryKey: noteKeys.all }),
);

/** `null` if `day` has no note. */
export const noteQueryOptions = (day: Day) =>
  queryOptions({
    queryKey: noteKeys.day(day),
    queryFn: () => invokeNotes({ name: "get_note", args: { day } }),
  });

/** Months having notes, most recent first. */
export const noteMonthsQueryOptions = queryOptions({
  queryKey: noteKeys.months(),
  queryFn: () => invokeNotes({ name: "list_note_months" }),
});

/** The notes of `month`, by day. */
export const monthNotesQueryOptions = (month: Month) =>
  queryOptions({
    queryKey: noteKeys.month(month),
    queryFn: () => {
      const [start, end] = monthBounds(month);
      return invokeNotes({ name: "list_notes_between", args: { start, end } });
    },
  });
