import type { UUID } from "node:crypto";
import { describe, expect, it } from "vitest";

import type { Workspace } from "../client/app";
import { pickedWorkspace } from "./workspaces";

const DATE = new Date("2026-10-10T12:00:00Z");

const workspace = (id: string, isActive = false): Workspace => ({
  id: id as UUID,
  name: id,
  path: `/${id}.sqlite3`,
  isActive,
  available: true,
  createdAt: DATE,
  autoBackup: true,
  autoBackupKeep: 7,
  linksAutoRefresh: true,
  linksSuggestDone: true,
  linksPasteToCreate: true,
});

describe("pickedWorkspace", () => {
  const work = workspace("work");
  const personal = workspace("personal", true);

  it("is the one picked in the URL", () => {
    expect(pickedWorkspace([work, personal], "work")).toBe(work);
  });

  it("else the active one, else the first one", () => {
    expect(pickedWorkspace([work, personal], null)).toBe(personal);
    expect(pickedWorkspace([work, personal], "removed")).toBe(personal);
    expect(pickedWorkspace([work, workspace("archive")], null)).toBe(work);
  });

  it("is null without any workspace", () => {
    expect(pickedWorkspace([], "work")).toBeNull();
  });
});
