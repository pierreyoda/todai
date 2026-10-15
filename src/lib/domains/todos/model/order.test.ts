import { describe, expect, it } from "vitest";

import { reorderTodos, reorderToEdge, sameOrder } from "./order";

const items = ["a", "b", "c", "d"].map((id) => ({ id }));
const ids = (list: readonly { id: string }[]) =>
  list.map(({ id }) => id).join("");

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
  ] as const)(
    "moves %s next to %s (%s): %s",
    (id, targetId, placement, expected) => {
      expect(ids(reorderTodos(items, id, targetId, placement))).toBe(expected);
    },
  );

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

describe("reorderToEdge", () => {
  // `x` and `y` are completed: in display order, among the others
  const todo = (id: string) => ({ id, completed: id === "x" || id === "y" });
  const todos = ["a", "x", "b", "c", "y"].map(todo);

  it.each([
    ["c", "first", "a", "before"],
    ["b", "first", "a", "before"],
    ["a", "last", "c", "after"],
    ["b", "last", "c", "after"],
    // Among the completed ones only
    ["y", "first", "x", "before"],
    ["x", "last", "y", "after"],
  ] as const)("moves %s %s: %s (%s)", (id, edge, targetId, placement) => {
    expect(reorderToEdge(todos, todo(id), edge)).toEqual({
      targetId,
      placement,
    });
  });

  it.each([
    ["a", "first"],
    ["c", "last"],
    ["x", "first"],
    ["y", "last"],
  ] as const)("leaves %s already %s", (id, edge) => {
    expect(reorderToEdge(todos, todo(id), edge)).toBeUndefined();
  });

  it("ignores a todo it isn't given", () => {
    expect(reorderToEdge(todos, todo("z"), "first")).toBeUndefined();
    expect(reorderToEdge([], todo("a"), "last")).toBeUndefined();
  });
});

describe("sameOrder", () => {
  it("compares the items' IDs in order", () => {
    expect(sameOrder(items, [...items])).toBe(true);
    expect(sameOrder(items, reorderTodos(items, "a", "b", "after"))).toBe(
      false,
    );
    expect(sameOrder(items, items.slice(1))).toBe(false);
  });
});
