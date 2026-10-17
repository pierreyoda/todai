import { describe, expect, it } from "vitest";

import {
  formatDue,
  notionDone,
  notionStatus,
  notionStatusWords,
} from "./notion";
import type { NotionDatabaseSnapshot, NotionPageSnapshot } from "./types";

const page = (
  overrides: Partial<NotionPageSnapshot> = {},
): NotionPageSnapshot => ({
  type: "notion_page",
  title: "Add links metadata to todos",
  uniqueId: { prefix: "TOD", number: 31 },
  status: { name: "In progress", color: "blue", group: "in_progress" },
  assignees: ["Pierre-Yves D."],
  tags: [],
  lastEditedAt: new Date("2026-10-15T10:12:00Z"),
  inTrash: false,
  ...overrides,
});

const database: NotionDatabaseSnapshot = {
  type: "notion_database",
  title: "Tickets",
  lastEditedAt: new Date("2026-10-14T08:00:00Z"),
  inTrash: false,
};

describe("notionStatus", () => {
  it("shows the page's Status in its color, with its group's glyph", () => {
    expect(notionStatus(page())).toEqual({
      word: "In progress",
      tone: "neutral",
      color: "blue",
      glyph: "dot",
    });
    expect(
      notionStatus(
        page({ status: { name: "Done", color: "green", group: "complete" } }),
      )?.glyph,
    ).toBe("check");
    expect(
      notionStatus(page({ status: { name: "Later", color: "gray" } }))?.glyph,
    ).toBeUndefined();
  });

  it("warns about trashed pages, and says nothing otherwise", () => {
    expect(notionStatus(page({ inTrash: true }))).toEqual({
      word: "In trash",
      tone: "warn",
    });
    expect(notionStatus(page({ status: undefined }))).toBeNull();
    expect(notionStatus(database)).toBeNull();
  });
});

describe("notionStatusWords", () => {
  it("lists the Status, then whether it's in the trash", () => {
    expect(
      notionStatusWords(page({ inTrash: true })).map((w) => w.word),
    ).toEqual(["In progress", "In trash"]);
    expect(notionStatusWords(database)).toEqual([]);
  });
});

describe("notionDone", () => {
  it("is done once the Status is in the Complete group", () => {
    expect(
      notionDone(
        page({
          status: { name: "Shipped", color: "purple", group: "complete" },
        }),
        "TOD-12",
      ),
    ).toEqual({ word: "Shipped", reason: "TOD-12 is Shipped in Notion" });
    expect(notionDone(page(), "TOD-31")).toBeUndefined();
    expect(
      notionDone(
        page({
          inTrash: true,
          status: { name: "Done", color: "green", group: "complete" },
        }),
        "TOD-31",
      ),
    ).toBeUndefined();
    expect(notionDone(database, "Tickets")).toBeUndefined();
  });
});

describe("formatDue", () => {
  it("formats days and date-times", () => {
    expect(formatDue("2026-10-16")).toBe("Fri, Oct 16");
    expect(formatDue("2026-10-16T14:30:00.000+02:00")).toMatch(
      /^Fri, Oct 16, \d{2}:\d{2}$/,
    );
    expect(formatDue("whenever")).toBe("whenever");
  });
});
