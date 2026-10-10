import type { UUID } from "node:crypto";

/** Format: YYYY-MM-DD */
export type Day = string;

/** Format: YYYY-MM */
export type Month = string;

/** A month's statistics about its non-deleted todos. */
export interface TodoMonth {
  month: Month;
  count: number;
  completedCount: number;
  estimatedMinutes: number;
  estimatedPoints: number;
}

/** A todo's estimate: either a duration or story points. */
export type Estimate =
  | {
      unit: "minutes";
      /** Positive integer, at most 24 × 60. */
      value: number;
    }
  | {
      unit: "points";
      /** Positive integer, at most 100. */
      value: number;
    };

export interface Todo {
  /** UUID v7. */
  id: UUID;
  day: Day;
  title: string;
  description?: string;
  completed: boolean;
  position: string;
  estimate?: Estimate;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
  deletedAt?: Date;
  /** IDs of its non-deleted tags, to resolve against the tags list (`list_tags`). */
  tagIds: readonly UUID[];
}

/** A day's note, in Markdown. */
export interface Note {
  day: Day;
  /** Never blank: a note emptied is deleted. */
  content: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface Tag {
  /** UUID v7. */
  id: UUID;
  name: string;
  /** Number of non-deleted todos with this tag. Only set by `list_tags`. */
  linkedTodosCount?: number;
  /** RGB color code. Example: "#FF0000". */
  color: string;
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}
