import { describe, expect, it } from "vitest";

import { notePath } from "./paths";

describe("notePath", () => {
  it("opens today's note as Today's sub-page", () => {
    expect(notePath("2026-10-12", "2026-10-12")).toBe("/today/notes");
  });

  it.each(["2026-10-11", "2025-12-31", "2026-10-13"])("opens the note of %s from the calendar", (day) => {
    expect(notePath(day, "2026-10-12")).toBe(`/calendar/notes/${day}`);
  });
});
