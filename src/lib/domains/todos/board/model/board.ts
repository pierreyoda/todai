import { dateToTodaiDate, formatDay } from "#core/time";
import { formatTime } from "#shared/utils/timestamps.ts";
import { reorderTodos, sameOrder } from "../../model/order";
import type { Placement, Todo } from "../../model/types";

/** The board's columns, from left to right: a todo is in exactly one of them (see `columnOf`). */
export const BOARD_COLUMNS = ["todo", "urgent", "doing", "done"] as const;
export type BoardColumn = (typeof BOARD_COLUMNS)[number];

/** What decides a todo's column. */
type BoardState = Pick<Todo, "completed" | "urgent" | "startedAt">;

/**
 * The column of `todo`: Done once completed, whatever else; else Doing once started; else Urgent if flagged; else
 * To-Do. Completing a todo keeps its flag and when it was started, so that reopened, it's back where it was.
 */
export const columnOf = ({
  completed,
  urgent,
  startedAt,
}: BoardState): BoardColumn => {
  if (completed) return "done";
  if (startedAt) return "doing";
  return urgent ? "urgent" : "todo";
};

/** `todos` (a day's, in display order) by column, each in that order. */
export const boardColumns = <T extends BoardState>(
  todos: readonly T[],
): Record<BoardColumn, T[]> => {
  const columns: Record<BoardColumn, T[]> = {
    todo: [],
    urgent: [],
    doing: [],
    done: [],
  };
  for (const todo of todos) columns[columnOf(todo)].push(todo);
  return columns;
};

/** What `update_todo` changes of a todo dropped on a column, from another one. */
export type ColumnChanges = {
  completed: boolean;
  urgent?: boolean;
  started?: boolean;
};

/**
 * To-Do and Urgent clear the flag or set it, and unstart the todo; Doing starts it, keeping its flag; Done completes
 * it, keeping both. Any column but Done reopens it.
 */
const COLUMN_CHANGES: Record<BoardColumn, ColumnChanges> = {
  todo: { completed: false, urgent: false, started: false },
  urgent: { completed: false, urgent: true, started: false },
  doing: { completed: false, started: true },
  done: { completed: true },
};

/** Next to which todo, and on which side, as `reorder_todo` takes it. */
type Neighbor = { targetId: Todo["id"]; placement: Placement };

/** Where a todo is dropped: on `column`, next to one of its cards (`next`) if dropped on it. */
export type BoardDrop = { column: BoardColumn; next?: Neighbor };

/** How a drop moves a todo: what changes if it leaves its column, and what it's put next to if its place changes. */
export type BoardMove = { changes?: ColumnChanges; next?: Neighbor };

/**
 * How dropping todo `id` moves it among `todos` (a day's, in display order): next to the card it's dropped on, else
 * after the column's last one. Columns show their todos in the day's order, as the list shows its completed ones apart:
 * it's only reordered in the day if its place in the column changes, and otherwise keeps it (e.g. in an empty column).
 * Without `reorder` (e.g. while a search hides some of them, among which it could land), it always keeps it.
 *
 * `null` if it changes nothing (e.g. dropped back where it was), or if it isn't among `todos`.
 */
export const planDrop = (
  todos: readonly Todo[],
  id: Todo["id"],
  { column, next }: BoardDrop,
  { reorder = true }: { reorder?: boolean } = {},
): BoardMove | null => {
  const todo = todos.find((other) => other.id === id);
  if (!todo) return null;
  const changes =
    columnOf(todo) === column ? undefined : COLUMN_CHANGES[column];
  const neighbor = reorder ? (next ?? lastOf(todos, column, id)) : null;
  /** The column once it's dropped there, in the day's order. */
  const inColumn = (day: readonly Todo[]) =>
    day.filter((other) => other.id === id || columnOf(other) === column);
  // Not when dropped on itself, nor right after the card before it: that would only reorder it among other columns
  const moves =
    !!neighbor &&
    !sameOrder(
      inColumn(todos),
      inColumn(reorderTodos(todos, id, neighbor.targetId, neighbor.placement)),
    );
  if (!changes && !moves) return null;
  return { changes, next: moves ? neighbor : undefined };
};

/** After the last todo of `column` but `id`, if any. */
const lastOf = (
  todos: readonly Todo[],
  column: BoardColumn,
  id: Todo["id"],
): Neighbor | null => {
  const last = todos.findLast(
    (todo) => todo.id !== id && columnOf(todo) === column,
  );
  return last ? { targetId: last.id, placement: "after" } : null;
};

/**
 * `todos` once todo `id` is moved so at `now`, as `update_todo` then `reorder_todo` store it: to show it right away.
 * `todos` itself is left unchanged.
 */
export const applyMove = (
  todos: readonly Todo[],
  id: Todo["id"],
  { changes, next }: BoardMove,
  now: Date,
): Todo[] => {
  const changed = todos.map((todo) =>
    todo.id === id && changes ? withChanges(todo, changes, now) : todo,
  );
  return next
    ? reorderTodos(changed, id, next.targetId, next.placement)
    : changed;
};

const withChanges = (
  todo: Todo,
  { completed, urgent, started }: ColumnChanges,
  now: Date,
): Todo => ({
  ...todo,
  completed,
  // Only when it changes, as `update_todo` does
  completedAt:
    completed === todo.completed
      ? todo.completedAt
      : completed
        ? now
        : undefined,
  urgent: urgent ?? todo.urgent,
  // Started twice, it keeps when it was first
  startedAt:
    started === undefined
      ? todo.startedAt
      : started
        ? (todo.startedAt ?? now)
        : undefined,
});

/** When a todo in Doing was started: its time if on `now`'s day, else its day. E.g. "10:42", or "Fri 16". */
export const formatStarted = (startedAt: Date, now: Date): string => {
  const day = dateToTodaiDate(startedAt);
  return day === dateToTodaiDate(now) ? formatTime(startedAt) : formatDay(day);
};
