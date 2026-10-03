import type { UUID } from "node:crypto";

/** Format: YYYY-MM-DD */
export type Day = string;

export interface Todo {
  /** UUID v7. */
  id: UUID;
  day: Day;
  title: string;
  description?: string;
  completed: boolean;
  position: string;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
  deletedAt?: Date;
  /** IDs of its non-deleted tags, to resolve against the tags list (`list_tags`). */
  tagIds: readonly UUID[];
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
