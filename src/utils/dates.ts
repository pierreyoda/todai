import type { Day } from "../client/types";

const padDatePart = (value: number, length: number): string => String(value).padStart(length, "0");

/**
 * Formats `date` as `YYYY-MM-DD`, in the local time zone.
 *
 * Not `date.toISOString().slice(0, 10)`, which gives the UTC date
 * (e.g. the previous day just after midnight in UTC+2).
 *
 * @throws {RangeError} If `date` is invalid, or its year is outside 0000-9999.
 */
export const dateToTodaiDate = (date: Date): Day => {
  const year = date.getFullYear();
  // Also rejects invalid dates, whose year is `NaN`.
  if (!(year >= 0 && year <= 9999)) {
    throw new RangeError(`Cannot format ${date} as YYYY-MM-DD`);
  }
  return [
    padDatePart(year, 4),
    padDatePart(date.getMonth() + 1, 2),
    padDatePart(date.getDate(), 2),
  ].join("-");
};
