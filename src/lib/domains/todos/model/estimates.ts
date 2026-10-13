import type { Estimate } from "./types";

/** @throws {RangeError} If `value` isn't a positive integer. */
const checkValue = (value: number, unit: Estimate["unit"]) => {
  if (!(Number.isInteger(value) && value > 0)) {
    throw new RangeError(`Invalid estimate: ${value} ${unit}`);
  }
};

/** E.g. "45m", "2h", or "1h 30m". */
const formatMinutes = (minutes: number): string => {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${rest}m`;
  return rest === 0 ? `${hours}h` : `${hours}h ${rest}m`;
};

/** E.g. "1 pt", or "3 pts". */
const formatPoints = (points: number): string =>
  `${points} ${points === 1 ? "pt" : "pts"}`;

/**
 * E.g. "1h 30m", or "3 pts".
 *
 * @throws {RangeError} If the estimate's value isn't a positive integer.
 */
export const formatEstimate = ({ unit, value }: Estimate): string => {
  checkValue(value, unit);
  return unit === "minutes" ? formatMinutes(value) : formatPoints(value);
};

/**
 * The total of `estimates` for each unit, e.g. "4h 15m · 13 pts", "2h", or "5 pts".
 * Todos without an estimate are skipped; `undefined` if none has one.
 *
 * @throws {RangeError} If an estimate's value isn't a positive integer.
 */
export const formatEstimates = (
  estimates: Iterable<Estimate | null | undefined>,
): string | undefined => {
  let minutes = 0;
  let points = 0;
  for (const estimate of estimates) {
    if (!estimate) continue;
    checkValue(estimate.value, estimate.unit);
    if (estimate.unit === "minutes") minutes += estimate.value;
    else points += estimate.value;
  }
  const parts = [
    minutes > 0 && formatMinutes(minutes),
    points > 0 && formatPoints(points),
  ].filter((part) => part !== false);
  return parts.length > 0 ? parts.join(" · ") : undefined;
};

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
