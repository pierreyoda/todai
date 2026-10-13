import type { UUID } from "node:crypto";

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
