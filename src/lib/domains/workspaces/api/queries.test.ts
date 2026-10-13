import { QueryClient } from "@tanstack/svelte-query";
import { describe, expect, it } from "vitest";

import { refreshGitHubAttention, workspaceKeys } from "./queries";

describe("refreshGitHubAttention", () => {
  it("refreshes the workspaces, but not what's nested in their keys", async () => {
    const id = "0199d0c0-0000-7000-8000-000000000001";
    const keys = {
      workspaces: workspaceKeys.all,
      active: workspaceKeys.active(),
      github: workspaceKeys.github(id),
      backups: workspaceKeys.backups(id),
    };
    const queryClient = new QueryClient();
    for (const key of Object.values(keys)) {
      queryClient.setQueryData(key, []);
    }

    await refreshGitHubAttention(queryClient);

    const invalidated = Object.fromEntries(
      Object.entries(keys).map(([name, key]) => [name, queryClient.getQueryState(key)?.isInvalidated]),
    );
    expect(invalidated).toEqual({ workspaces: true, active: true, github: false, backups: false });
  });
});
