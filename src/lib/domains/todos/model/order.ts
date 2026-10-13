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
  others.splice(placement === "before" ? targetIndex : targetIndex + 1, 0, moved);
  return others;
};

/** Whether `a` and `b` list the same items in the same order. */
export const sameOrder = <T extends { id: string }>(a: readonly T[], b: readonly T[]) =>
  a.length === b.length && a.every((item, index) => item.id === b[index].id);
