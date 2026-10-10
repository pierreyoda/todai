import type { UUID } from "node:crypto";
import { describe, expect, it } from "vitest";

import type { Tag, Todo } from "../client/types";
import { createTodoSearch, filterTodos } from "./todoSearch";

const DATE = new Date("2026-10-01T00:00:00Z");

const tag = (id: string, name: string): Tag => ({
  id: id as UUID,
  name,
  color: "#ec4899",
  createdAt: DATE,
  updatedAt: DATE,
});

const todo = (
  id: string,
  title: string,
  { tagIds = [], description }: { tagIds?: string[]; description?: string } = {},
): Todo => ({
  id: id as UUID,
  day: "2026-10-01",
  title,
  description,
  completed: false,
  position: id,
  createdAt: DATE,
  updatedAt: DATE,
  tagIds: tagIds as UUID[],
});

const TAGS = [tag("work", "work"), tag("urgent", "urgent")];
const TODOS = [
  todo("report", "Write the quarterly report for the board", {
    tagIds: ["work", "urgent"],
  }),
  todo("bank", "Call the bank", { description: "About the mortgage" }),
  todo("milk", "Buy milk"),
  todo("event", "Préparer l'événement"),
  todo("cafe", "Cafe with Sam", { tagIds: ["work"] }),
];

const search = (query: string, todos = TODOS, tags = TAGS) => {
  const matches = createTodoSearch(todos, tags)(query);
  return matches && [...matches.keys()];
};

describe("createTodoSearch", () => {
  it.each(["", "   "])("doesn't filter for the blank query %j", (query) => {
    expect(search(query)).toBeNull();
  });

  it.each([
    ["report", "an exact word"],
    ["repport", "a typo"],
    ["board", "a word at the end of a long title"],
    ["QUARTERLY", "another case"],
  ])("finds a title by %j (%s)", (query) => {
    expect(search(query)).toEqual(["report"]);
  });

  it("finds nothing for unrelated words", () => {
    expect(search("dentist")).toEqual([]);
  });

  it("doesn't take a word for a similar one", () => {
    // "reparer" is two letters away from "report"
    expect(search("report")).not.toContain("event");
  });

  it.each([
    ["evenement", "event"],
    ["café", "cafe"],
  ])("ignores diacritics: %j finds %j", (query, expected) => {
    expect(search(query)).toEqual([expected]);
  });

  it("finds todos by the names of their tags", () => {
    expect(search("urgent")).toEqual(["report"]);
  });

  it("finds tags by their current name", () => {
    const renamed = [tag("work", "office"), tag("urgent", "urgent")];
    expect(search("office", TODOS, renamed)?.sort()).toEqual(["cafe", "report"]);
    expect(search("work", TODOS, renamed)).toEqual([]);
  });

  it("finds todos by their description", () => {
    expect(search("mortgage")).toEqual(["bank"]);
  });

  it("gives the ranges of the title matching the query", () => {
    const matches = createTodoSearch(TODOS, TAGS)("milk");
    expect(matches?.get("milk" as UUID)?.titleRanges).toEqual([[4, 7]]);
  });

  it("gives no title ranges when only another field matches", () => {
    const matches = createTodoSearch(TODOS, TAGS)("mortgage");
    expect(matches?.get("bank" as UUID)?.titleRanges).toEqual([]);
  });
});

describe("filterTodos", () => {
  it("keeps all the todos without a search", () => {
    expect(filterTodos(TODOS, null)).toBe(TODOS);
  });

  it("keeps the matching todos, in their own order", () => {
    // Listed best match first by the search
    const matches = createTodoSearch(TODOS, TAGS)("work");
    expect(filterTodos(TODOS, matches).map(({ id }) => id)).toEqual([
      "report",
      "cafe",
    ]);
  });
});
