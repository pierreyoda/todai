import { describe, expect, it } from "vitest";

import { NoteDrafts } from "./noteDrafts.svelte";

const DAY = "2026-10-12";

describe("NoteDrafts", () => {
  it.each([true, false])(
    "tells whether a day not being written has a note as saved (%s)",
    (saved) => {
      expect(new NoteDrafts().hasNote(DAY, saved)).toBe(saved);
    },
  );

  it("follows the draft of a day being written, rather than its saved note", () => {
    const drafts = new NoteDrafts();
    let draft = "";
    drafts.open(DAY, () => draft);
    expect(drafts.hasNote(DAY, true)).toBe(false);

    draft = "# Today";
    expect(drafts.hasNote(DAY, false)).toBe(true);
    expect(drafts.hasNote("2026-10-11", false)).toBe(false);
  });

  it("counts a blank draft as no note, as saving deletes it", () => {
    const drafts = new NoteDrafts();
    drafts.open(DAY, () => "  \n\t ");
    expect(drafts.hasNote(DAY, true)).toBe(false);
  });

  it("goes back to the saved note once closed", () => {
    const drafts = new NoteDrafts();
    const close = drafts.open(DAY, () => "");
    close();
    expect(drafts.hasNote(DAY, true)).toBe(true);
  });

  it("keeps following a newer editor of the same day when an older one closes", () => {
    const drafts = new NoteDrafts();
    const closeOlder = drafts.open(DAY, () => "");
    drafts.open(DAY, () => "Newer");
    closeOlder();
    expect(drafts.hasNote(DAY, false)).toBe(true);
  });
});
