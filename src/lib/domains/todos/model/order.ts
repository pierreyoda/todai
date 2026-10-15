import type { Placement } from "./types";

/**
 * `items` with the one `id` moved right before or after `targetId`, as `reorder_todo` does with the todos of a day: e.g.
 * to show it right away. In the same order if either isn't in `items`, or if they're the same.
 */
export const reorderTodos = <T extends { id: string }>(
  items: readonly T[],
  id: T["id"],
  targetId: T["id"],
  placement: Placement,
): T[] => {
  const moved = items.find((item) => item.id === id);
  const others = items.filter((item) => item.id !== id);
  const targetIndex = others.findIndex((item) => item.id === targetId);
  if (!moved || targetIndex === -1) return [...items];
  others.splice(
    placement === "before" ? targetIndex : targetIndex + 1,
    0,
    moved,
  );
  return others;
};

/**
 * How `reorder_todo` moves `todo` first or last among the todos as completed as it, which are listed and reordered apart:
 * next to which one of `todos` (a day's, in display order), and on which side. `undefined` if it's already there, or
 * isn't among them.
 */
export const reorderToEdge = <T extends { id: string; completed: boolean }>(
  todos: readonly T[],
  todo: T,
  edge: "first" | "last",
): { targetId: T["id"]; placement: Placement } | undefined => {
  const group = todos.filter(({ completed }) => completed === todo.completed);
  if (!group.some(({ id }) => id === todo.id)) return undefined;
  const target = edge === "first" ? group.at(0) : group.at(-1);
  if (!target || target.id === todo.id) return undefined;
  return {
    targetId: target.id,
    placement: edge === "first" ? "before" : "after",
  };
};

/** Whether `a` and `b` list the same items in the same order. */
export const sameOrder = <T extends { id: string }>(
  a: readonly T[],
  b: readonly T[],
) => a.length === b.length && a.every((item, index) => item.id === b[index].id);
