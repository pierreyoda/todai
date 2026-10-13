/** E.g. "5 todos · 3 done". */
export const formatTodoCounts = (count: number, completedCount: number): string =>
  `${count} ${count === 1 ? "todo" : "todos"} · ${completedCount} done`;
