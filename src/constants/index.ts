export const TODO_ESTIMATE_MINUTES_PRESETS: readonly {
  value: number;
}[] = [
    { value: 5 },
    { value: 15 },
    { value: 30 },
    { value: 45 },
    { value: 60 },
    { value: 120 },
    { value: 240 },
    { value: 480 },
  ];

export const TODO_ESTIMATE_POINTS_PRESET: readonly {
  value: number;
}[] = [
    { value: 1 },
    { value: 2 },
    { value: 3 },
    { value: 5 },
    { value: 8 },
    { value: 13 },
    { value: 21 },
    { value: 34 },
    { value: 55 },
    { value: 89 },
  ];

/** How many days before today the todos left unfinished are offered to be moved to today. */
export const UNFINISHED_TODOS_LOOKBACK_DAYS = 7;
