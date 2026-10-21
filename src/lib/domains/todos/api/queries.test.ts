import type { UUID } from "node:crypto";
import { QueryClient, type QueryKey } from "@tanstack/svelte-query";
import { describe, expect, it } from "vitest";

import type { Day } from "#core/time";
import { tagKeys } from "#domains/tags";
import type { Todo, TodoMonth } from "../model/types";
import {
  invalidateTodosOf,
  setCachedTodoDescription,
  todoKeys,
} from "./queries";

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
    urgent: false,
    tagIds: [],
    links: [],
  });

  const edited = todo("edited", "2026-10-03", "Before");
  const sameDay = todo("same-day", "2026-10-03");
  const otherDay = todo("other-day", "2026-10-04");
  const otherMonth = todo("other-month", "2026-09-30");
  const months: TodoMonth[] = [
    {
      month: "2026-10",
      count: 3,
      completedCount: 0,
      estimatedMinutes: 0,
      estimatedPoints: 0,
    },
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
    for (const name of [
      "otherDay",
      "otherMonth",
      "otherUnfinished",
      "months",
    ]) {
      expect(data(name), name).toBe(lists[name][1]);
    }
  });
});
