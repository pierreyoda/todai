import { QueryClient } from "@tanstack/svelte-query";
import { describe, expect, it } from "vitest";

import { noteKeys } from "./queries";

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
      Object.entries(keys).map(([name, key]) => [
        name,
        queryClient.getQueryState(key)?.isInvalidated,
      ]),
    );
    expect(invalidated).toEqual({ months: true, month: true, day: false });
  });
});
