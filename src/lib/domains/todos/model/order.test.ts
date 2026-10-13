import { describe, expect, it } from "vitest";

import { reorderTodos, sameOrder } from "./order";

const items = ["a", "b", "c", "d"].map((id) => ({ id }));
const ids = (list: readonly { id: string }[]) => list.map(({ id }) => id).join("");

describe("reorderTodos", () => {
  it.each([
    ["d", "b", "before", "adbc"],
    ["a", "c", "after", "bcad"],
    ["d", "a", "before", "dabc"],
    ["a", "d", "after", "bcda"],
    // Past its neighbor
    ["a", "b", "after", "bacd"],
    ["c", "b", "before", "acbd"],
    // Already there
    ["b", "c", "before", "abcd"],
    ["c", "b", "after", "abcd"],
  ] as const)("moves %s next to %s (%s): %s", (id, targetId, placement, expected) => {
    expect(ids(reorderTodos(items, id, targetId, placement))).toBe(expected);
  });

  it.each([
    ["itself", "b", "b"],
    ["an unknown todo", "x", "b"],
    ["an unknown target", "b", "x"],
  ])("keeps the order for %s", (_, id, targetId) => {
    expect(ids(reorderTodos(items, id, targetId, "after"))).toBe("abcd");
  });

  it("leaves the list it's given unchanged", () => {
    reorderTodos(items, "a", "d", "after");
    expect(ids(items)).toBe("abcd");
  });
});

describe("sameOrder", () => {
  it("compares the items' IDs in order", () => {
    expect(sameOrder(items, [...items])).toBe(true);
    expect(sameOrder(items, reorderTodos(items, "a", "b", "after"))).toBe(false);
    expect(sameOrder(items, items.slice(1))).toBe(false);
  });
});
