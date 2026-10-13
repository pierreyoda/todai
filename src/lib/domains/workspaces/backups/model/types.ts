import type { UUID } from "node:crypto";

/**
 * What a backup was made for:
 * - `manual`: asked for;
 * - `automatic`: made when the workspace is opened, at most once a day (only the most recent ones are kept);
 * - `pre_migration`: made before migrating its database to a newer version of todai;
 * - `pre_restore`: its state before restoring another backup, so that restoring can be undone.
 */
export type WorkspaceBackupKind = "manual" | "automatic" | "pre_migration" | "pre_restore";

/** A backup of a workspace's database, kept in the app's data directory. */
export interface WorkspaceBackup {
  /** UUID v7. */
  id: UUID;
  workspaceId: UUID;
  /** Absolute path of its file. */
  path: string;
  kind: WorkspaceBackupKind;
  /** Whether its file still exists (e.g. not deleted by hand). */
  available: boolean;
  /** Of its file, in bytes, if available. */
  size?: number;
  createdAt: Date;
}
