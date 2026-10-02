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
  /** Associated tags, sorted by name. */
  tags: readonly Tag[];
}

export interface Tag {
  /** UUID v7. */
  id: UUID;
  name: string;
  /** RGB color code. Example: "#FF0000". */
  color: string;
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}
