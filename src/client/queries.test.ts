import { QueryClient } from "@tanstack/svelte-query";
import { describe, expect, it } from "vitest";

import { invalidateTodosOf, tagKeys, todoKeys } from "./queries";

describe("invalidateTodosOf", () => {
  const keys = {
    sameDay: todoKeys.day("2026-10-03"),
    sameMonthDay: todoKeys.day("2026-10-31"),
    sameMonth: todoKeys.month("2026-10"),
    months: todoKeys.months(),
    otherMonth: todoKeys.month("2026-09"),
    otherMonthDay: todoKeys.day("2026-09-30"),
    tags: tagKeys.all,
  };

  it("refreshes the day's month, its days and the months statistics only", async () => {
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
      tags: false,
    });
  });
});
