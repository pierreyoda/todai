import type { UUID } from "node:crypto";
import { QueryClient, type QueryKey } from "@tanstack/svelte-query";
import { describe, expect, it } from "vitest";

import {
  clearActiveWorkspace,
  ghKeys,
  invalidateActiveWorkspace,
  invalidateTodosOf,
  linkKeys,
  linkStatusKeys,
  noteKeys,
  refreshGitHubAttention,
  setCachedTodoDescription,
  todoKeys,
  workspaceKeys,
} from "./queries";
import { tagKeys } from "#domains/tags";
import type { Day, Todo, TodoMonth } from "./types";

describe("invalidateTodosOf", () => {
  const keys = {
    sameDay: todoKeys.day("2026-10-03"),
    sameMonthDay: todoKeys.day("2026-10-31"),
    sameMonth: todoKeys.month("2026-10"),
    months: todoKeys.months(),
    otherMonth: todoKeys.month("2026-09"),
    otherMonthDay: todoKeys.day("2026-09-30"),
    unfinished: todoKeys.unfinishedBefore("2026-11-02"),
    tags: tagKeys.all,
  };

  it("refreshes the day's month, its days, the months statistics and the unfinished todos only", async () => {
    const queryClient = new QueryClient();
    for (const key of Object.values(keys)) {
      queryClient.setQueryData(key, []);
    }

    await invalidateTodosOf(queryClient, "2026-10-03");

    const invalidated = Object.fromEntries(
      Object.entries(keys).map(([name, key]) => [
        name,
        queryClient.getQueryState(key)?.isInvalidated,
      ]),
    );
    expect(invalidated).toEqual({
      sameDay: true,
      sameMonthDay: true,
      sameMonth: true,
      months: true,
      otherMonth: false,
      otherMonthDay: false,
      // Possibly from another month than the day's
      unfinished: true,
      tags: false,
    });
  });
});

describe("setCachedTodoDescription", () => {
  const DATE = new Date("2026-10-01T00:00:00Z");

  const todo = (id: string, day: Day, description?: string): Todo => ({
    id: id as UUID,
    day,
    title: id,
    description,
    completed: false,
    position: id,
    createdAt: DATE,
    updatedAt: DATE,
    tagIds: [],
    links: [],
  });

  const edited = todo("edited", "2026-10-03", "Before");
  const sameDay = todo("same-day", "2026-10-03");
  const otherDay = todo("other-day", "2026-10-04");
  const otherMonth = todo("other-month", "2026-09-30");
  const months: TodoMonth[] = [
    { month: "2026-10", count: 3, completedCount: 0, estimatedMinutes: 0, estimatedPoints: 0 },
  ];

  /** The cached lists, by name: the edited todo's, then others. */
  const withLists = () => {
    const lists: Record<string, [QueryKey, unknown]> = {
      day: [todoKeys.day("2026-10-03"), [edited, sameDay]],
      month: [todoKeys.month("2026-10"), [edited, sameDay, otherDay]],
      unfinished: [todoKeys.unfinishedBefore("2026-10-05"), [edited, otherDay]],
      otherDay: [todoKeys.day("2026-10-04"), [otherDay]],
      otherMonth: [todoKeys.month("2026-09"), [otherMonth]],
      otherUnfinished: [todoKeys.unfinishedBefore("2026-10-01"), [otherMonth]],
      months: [todoKeys.months(), months],
    };
    const queryClient = new QueryClient();
    for (const [key, data] of Object.values(lists)) {
      queryClient.setQueryData(key, data);
    }
    const data = (name: string) => queryClient.getQueryData(lists[name][0]);
    return { queryClient, lists, data };
  };

  it("sets it in the lists holding the todo", () => {
    const { queryClient, data } = withLists();

    setCachedTodoDescription(queryClient, edited, "After");

    const after = { ...edited, description: "After" };
    expect(data("day")).toEqual([after, sameDay]);
    expect(data("month")).toEqual([after, sameDay, otherDay]);
    expect(data("unfinished")).toEqual([after, otherDay]);
  });

  it("removes it with `undefined`", () => {
    const { queryClient, data } = withLists();

    setCachedTodoDescription(queryClient, edited, undefined);

    expect((data("day") as Todo[])[0].description).toBeUndefined();
  });

  it("leaves the other todos and the lists without the todo as they are", () => {
    const { queryClient, lists, data } = withLists();

    setCachedTodoDescription(queryClient, edited, "After");

    expect((data("day") as Todo[])[1]).toBe(sameDay);
    for (const name of ["otherDay", "otherMonth", "otherUnfinished", "months"]) {
      expect(data(name), name).toBe(lists[name][1]);
    }
  });
});

describe("noteKeys", () => {
  it("refreshes the months' notes with their list, but not the days' notes", async () => {
    const keys = {
      months: noteKeys.months(),
      month: noteKeys.month("2026-10"),
      day: noteKeys.day("2026-10-10"),
    };
    const queryClient = new QueryClient();
    for (const key of Object.values(keys)) {
      queryClient.setQueryData(key, []);
    }

    await queryClient.invalidateQueries({ queryKey: noteKeys.months() });

    const invalidated = Object.fromEntries(
      Object.entries(keys).map(([name, key]) => [name, queryClient.getQueryState(key)?.isInvalidated]),
    );
    expect(invalidated).toEqual({ months: true, month: true, day: false });
  });
});

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
