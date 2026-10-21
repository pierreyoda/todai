import type { UUID } from "node:crypto";

import type { Day, Month } from "#core/time";
import type { TodoLink } from "../links/model/types";

/** A month's statistics about its non-deleted todos. */
export interface TodoMonth {
  month: Month;
  count: number;
  completedCount: number;
  estimatedMinutes: number;
  estimatedPoints: number;
}

/** Where a todo is put next to another one, e.g. when reordering them. */
export type Placement = "before" | "after";

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
  /** Shown as such, and in the board's "Urgent" column until started. */
  urgent: boolean;
  /**
   * When it was started, i.e. put in the board's "Doing" column. Kept once completed, so that reopened, it's back
   * there.
   */
  startedAt?: Date;
  /** IDs of its non-deleted tags, to resolve against the tags list (`list_tags`). */
  tagIds: readonly UUID[];
  /** In the order they were added. */
  links: readonly TodoLink[];
}
