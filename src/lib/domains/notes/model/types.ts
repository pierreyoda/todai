import type { Day } from "#core/time";

/** A day's note, in Markdown. */
export interface Note {
  day: Day;
  /** Never blank: a note emptied is deleted. */
  content: string;
  createdAt: Date;
  updatedAt: Date;
}
