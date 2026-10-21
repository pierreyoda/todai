import type { Todo } from "../../model/types";

/** What's dragged on the board: as plain data, as it's serialized during a drag. Its title is told from the keyboard. */
export type DraggedTodo = Pick<Todo, "id" | "title">;

/** The board's cards and columns: dropped only among each other, not on the list's rows. */
export const BOARD_DROP_GROUP = "board-todos";
