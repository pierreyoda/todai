import type { Day, Month } from "#core/time";
import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Note } from "../model/types";

type NoteCommands = {
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

export const invokeNotes = createInvoker<NoteCommands>("command");
