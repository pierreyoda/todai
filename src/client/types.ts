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
}
