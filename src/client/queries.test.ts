import { QueryClient } from "@tanstack/svelte-query";
import { describe, expect, it } from "vitest";

import { clearActiveWorkspace, invalidateActiveWorkspace, workspaceKeys } from "#domains/workspaces";
import { linkKeys, linkStatusKeys, todoKeys } from "#domains/todos";
import { noteKeys } from "#domains/notes";
import { tagKeys } from "#domains/tags";
import { ghKeys } from "./queries";

describe("active workspace changes", () => {
  /** The queries of the active workspace's database, then the workspaces. */
  const keys = {
    todos: todoKeys.day("2026-10-10"),
    months: todoKeys.months(),
    tags: tagKeys.all,
    note: noteKeys.day("2026-10-10"),
    linkStatuses: linkStatusKeys.day("2026-10-10"),
    workspaces: workspaceKeys.all,
  };

  const withData = () => {
    const queryClient = new QueryClient();
    for (const key of Object.values(keys)) {
      queryClient.setQueryData(key, []);
    }
    return queryClient;
  };

  it("refreshes everything from the workspace's database when switching to another one", async () => {
    const queryClient = withData();

    await invalidateActiveWorkspace(queryClient);

    for (const [name, key] of Object.entries(keys)) {
      expect(queryClient.getQueryState(key)?.isInvalidated, name).toBe(true);
    }
  });

  it("drops everything from the workspace's database when closing it", async () => {
    const queryClient = withData();

    await clearActiveWorkspace(queryClient);

    const { workspaces, ...fromDatabase } = keys;
    for (const [name, key] of Object.entries(fromDatabase)) {
      expect(queryClient.getQueryState(key), name).toBeUndefined();
    }
    expect(queryClient.getQueryState(workspaces)?.isInvalidated).toBe(true);
  });

  it.each([
    ["the recognized links", linkKeys.parse("https://github.com/pierreyoda/todai/pull/58")],
    ["the GitHub CLI status", ghKeys.status()],
  ])("keeps %s, which don't come from the workspace's database", async (_, key) => {
    const queryClient = withData();
    queryClient.setQueryData(key, {});

    await invalidateActiveWorkspace(queryClient);
    expect(queryClient.getQueryState(key)?.isInvalidated).toBe(false);

    await clearActiveWorkspace(queryClient);
    expect(queryClient.getQueryState(key)?.isInvalidated).toBe(false);
  });
});
