import { createMutation, useQueryClient } from "@tanstack/svelte-query";

import type { Day } from "#core/time";
import { invokeNotes } from "./commands";
import { noteKeys } from "./queries";

/**
 * Saves `content` as the note of `day`, as typed, or deletes it if blank. Must be called during component
 * initialization.
 *
 * The day is passed with each save, rather than read once saved: a slow save still updates the note it was made for.
 */
export const createSaveNoteMutation = () => {
  const queryClient = useQueryClient();
  return createMutation(() => ({
    mutationFn: ({ day, content }: { day: Day; content: string }) =>
      invokeNotes({ name: "save_note", args: { day, content } }),
    // The saved note (or `null`, once deleted) replaces the cached one, rather than being refetched. The calendar's
    // notes are only marked stale: refetched once shown, rather than at each autosave.
    onSuccess: (note, { day }) => {
      queryClient.setQueryData(noteKeys.day(day), note);
      void queryClient.invalidateQueries({
        queryKey: noteKeys.months(),
        refetchType: "none",
      });
    },
  }));
};
