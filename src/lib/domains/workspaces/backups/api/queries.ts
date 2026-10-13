import type { UUID } from "node:crypto";
import { queryOptions } from "@tanstack/svelte-query";

import { workspaceKeys } from "../../api/queries";
import { invokeBackups } from "./commands";

/** Most recent first. */
export const workspaceBackupsQueryOptions = (workspaceId: UUID) =>
  queryOptions({
    queryKey: workspaceKeys.backups(workspaceId),
    queryFn: () => invokeBackups({ name: "list_workspace_backups", args: { workspaceId } }),
  });
