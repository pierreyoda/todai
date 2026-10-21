import type { UUID } from "node:crypto";
import { describe, expect, it } from "vitest";

import type { Todo } from "../../model/types";
import {
  applyMove,
  type BoardColumn,
  type BoardDrop,
  boardColumns,
  columnOf,
  formatStarted,
  planDrop,
} from "./board";

const DATE = new Date("2026-10-17T08:00:00Z");
const STARTED = new Date("2026-10-17T10:42:00Z");
const NOW = new Date("2026-10-17T15:00:00Z");

const todo = (
  id: string,
  state: Partial<Pick<Todo, "completed" | "urgent" | "startedAt">> = {},
): Todo => ({
  id: id as UUID,
  day: "2026-10-17",
  title: id,
  completed: false,
  position: id,
  createdAt: DATE,
  updatedAt: DATE,
  urgent: false,
  tagIds: [],
  links: [],
  ...state,
  ...(state.completed ? { completedAt: DATE } : {}),
});

/** A day, in display order: two todos in each column, interleaved. */
const DAY = [
  todo("a"),
  todo("u1", { urgent: true }),
  todo("d1", { startedAt: STARTED }),
  todo("x1", { completed: true }),
  todo("b"),
  todo("u2", { urgent: true }),
  todo("d2", { startedAt: STARTED, urgent: true }),
  todo("x2", { completed: true, startedAt: STARTED, urgent: true }),
];

const id = (raw: string) => raw as UUID;
const ids = (todos: readonly Todo[]) => todos.map((t) => t.id).join(" ");
const find = (todos: readonly Todo[], raw: string) =>
  todos.find((t) => t.id === raw) as Todo;
const placedAfter = (targetId: string) =>
  ({ targetId: id(targetId), placement: "after" }) as const;
const placedBefore = (targetId: string) =>
  ({ targetId: id(targetId), placement: "before" }) as const;

describe("columnOf", () => {
  it.each([
    [{}, "todo"],
    [{ urgent: true }, "urgent"],
    [{ startedAt: STARTED }, "doing"],
    // Its flag follows it into Doing
    [{ startedAt: STARTED, urgent: true }, "doing"],
    // Done whatever else, which it keeps for when it's reopened
    [{ completed: true }, "done"],
    [{ completed: true, urgent: true, startedAt: STARTED }, "done"],
  ] as const)("puts %o in %s", (state, column) => {
    expect(columnOf(todo("t", state))).toBe(column);
  });
});

describe("boardColumns", () => {
  it("groups a day's todos by column, each in the day's order", () => {
    const columns = boardColumns(DAY);
    expect({
      todo: ids(columns.todo),
      urgent: ids(columns.urgent),
      doing: ids(columns.doing),
      done: ids(columns.done),
    }).toEqual({ todo: "a b", urgent: "u1 u2", doing: "d1 d2", done: "x1 x2" });
  });

  it("has every column, even empty", () => {
    expect(boardColumns([])).toEqual({
      todo: [],
      urgent: [],
      doing: [],
      done: [],
    });
  });
});

describe("planDrop", () => {
  it.each<[BoardColumn, string, object]>([
    ["todo", "x2", { completed: false, urgent: false, started: false }],
    ["urgent", "x2", { completed: false, urgent: true, started: false }],
    // Keeps its flag
    ["doing", "x2", { completed: false, started: true }],
    // Keeps its flag, and when it was started
    ["done", "d2", { completed: true }],
  ])(
    "changes a todo dropped on %s as the column says",
    (column, from, changes) => {
      expect(planDrop(DAY, id(from), { column })?.changes).toEqual(changes);
    },
  );

  it("puts a todo dropped on a column after its last card", () => {
    expect(planDrop(DAY, id("a"), { column: "doing" })).toEqual({
      changes: { completed: false, started: true },
      next: placedAfter("d2"),
    });
  });

  it("puts a todo dropped on a card next to it", () => {
    expect(
      planDrop(DAY, id("d2"), { column: "urgent", next: placedBefore("u1") }),
    ).toEqual({
      changes: { completed: false, urgent: true, started: false },
      next: placedBefore("u1"),
    });
  });

  it("keeps the place in the day of a todo dropped on an empty column", () => {
    const day = DAY.filter((t) => columnOf(t) !== "doing");
    expect(planDrop(day, id("a"), { column: "doing" })).toEqual({
      changes: { completed: false, started: true },
    });
  });

  it("keeps the place in the day of a todo already after the column's cards", () => {
    expect(planDrop(DAY, id("x2"), { column: "todo" })).toEqual({
      changes: { completed: false, urgent: false, started: false },
    });
  });

  it("only reorders a todo dropped in its own column", () => {
    expect(
      planDrop(DAY, id("b"), { column: "todo", next: placedBefore("a") }),
    ).toEqual({ next: placedBefore("a") });
    expect(planDrop(DAY, id("a"), { column: "todo" })).toEqual({
      next: placedAfter("b"),
    });
  });

  it.each<[string, BoardDrop]>([
    ["on itself", { column: "todo", next: placedBefore("b") }],
    // Which would only move it among other columns' todos in the day
    [
      "right after the card before it",
      { column: "todo", next: placedAfter("a") },
    ],
    ["on its column, whose last card it is", { column: "todo" }],
  ])("changes nothing for a todo dropped back where it was: %s", (_, drop) => {
    expect(planDrop(DAY, id("b"), drop)).toBeNull();
  });

  it("changes nothing for a todo it doesn't know", () => {
    expect(planDrop(DAY, id("unknown"), { column: "done" })).toBeNull();
  });

  it("only changes the column of a todo dropped without reordering", () => {
    const keep = { reorder: false };
    expect(planDrop(DAY, id("a"), { column: "doing" }, keep)).toEqual({
      changes: { completed: false, started: true },
    });
    const ownColumn = { column: "todo", next: placedBefore("a") } as const;
    expect(planDrop(DAY, id("b"), ownColumn, keep)).toBeNull();
  });
});

describe("applyMove", () => {
  const move = (raw: string, drop: BoardDrop) => {
    const planned = planDrop(DAY, id(raw), drop);
    if (!planned) throw new Error(`Dropping ${raw} changes nothing`);
    return applyMove(DAY, id(raw), planned, NOW);
  };

  it("starts a todo put in Doing, keeping its flag, at its new place", () => {
    const moved = move("u1", { column: "doing" });
    expect(find(moved, "u1")).toMatchObject({
      completed: false,
      urgent: true,
      startedAt: NOW,
    });
    expect(ids(boardColumns(moved).doing)).toBe("d1 d2 u1");
  });

  it("completes a todo put in Done, keeping its flag and when it was started", () => {
    expect(find(move("d2", { column: "done" }), "d2")).toMatchObject({
      completed: true,
      completedAt: NOW,
      urgent: true,
      startedAt: STARTED,
    });
  });

  it("reopens a todo taken out of Done, as started as it first was", () => {
    const reopened = find(move("x2", { column: "doing" }), "x2");
    expect(reopened).toMatchObject({
      completed: false,
      urgent: true,
      startedAt: STARTED,
    });
    expect(reopened.completedAt).toBeUndefined();
  });

  it("clears the flag and the start of a todo put back in To-Do", () => {
    const back = find(move("d2", { column: "todo" }), "d2");
    expect(back).toMatchObject({ completed: false, urgent: false });
    expect(back.startedAt).toBeUndefined();
  });

  it("only reorders a todo moved in its own column", () => {
    const moved = move("b", { column: "todo", next: placedBefore("a") });
    expect(ids(moved)).toBe("b a u1 d1 x1 u2 d2 x2");
    expect(find(moved, "b")).toBe(find(DAY, "b"));
  });

  it("leaves the given todos unchanged", () => {
    move("a", { column: "done" });
    expect(ids(DAY)).toBe("a u1 d1 x1 b u2 d2 x2");
    expect(find(DAY, "a").completed).toBe(false);
  });
});

describe("formatStarted", () => {
  it("tells the time of a todo started on the same day", () => {
    expect(formatStarted(STARTED, NOW)).toBe("10:42");
  });

  it("tells the day of a todo started on another one", () => {
    expect(formatStarted(new Date("2026-10-16T18:00:00Z"), NOW)).toBe("Fri 16");
  });
});
