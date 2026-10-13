import { describe, expect, it } from "vitest";

import { formatTodoCounts } from "./counts";

describe("formatTodoCounts", () => {
  it.each([
    [5, 3, "5 todos · 3 done"],
    [1, 0, "1 todo · 0 done"],
    [0, 0, "0 todos · 0 done"],
  ])("formats %i todos with %i done as %j", (count, completedCount, label) => {
    expect(formatTodoCounts(count, completedCount)).toBe(label);
  });
});
