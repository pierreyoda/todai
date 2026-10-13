import type { Workspace } from "./types";

/**
 * The workspace a settings page is about: the one picked in its URL (`?workspace=<id>`), else the active one, else the
 * first one. Even without a usable active workspace, e.g. to restore one of its backups. `null` without any.
 */
export const pickedWorkspace = (
  workspaces: readonly Workspace[],
  id: string | null,
): Workspace | null =>
  workspaces.find((workspace) => workspace.id === id) ??
  workspaces.find((workspace) => workspace.isActive) ??
  workspaces[0] ??
  null;
