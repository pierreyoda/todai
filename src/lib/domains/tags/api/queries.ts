import type { UUID } from "node:crypto";
import { queryOptions } from "@tanstack/svelte-query";

import { workspaceDataKey } from "#shared/ipc/queryClient.ts";
import { invokeTags } from "./commands";

export const tagKeys = {
  all: [...workspaceDataKey, "tags"] as const,
  id: (id: UUID) => [...tagKeys.all, id] as const,
};

export const tagsQueryOptions = queryOptions({
  queryKey: tagKeys.all,
  queryFn: () => invokeTags({ name: "list_tags" }),
});
