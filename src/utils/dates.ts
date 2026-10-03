import { utc } from "@date-fns/utc";
import {
  addMonths as addMonthsToDate,
  eachMonthOfInterval,
  eachWeekOfInterval,
  endOfMonth,
  endOfWeek,
  format,
  getYear,
  isSameDay,
  isSameMonth,
  isSameYear,
  isValid,
  max,
  min,
  parseISO,
  startOfMonth,
  startOfWeek,
} from "date-fns";

import type { Day, Month } from "../client/types";

/** A week of a month: Monday to Sunday, cut at the month's bounds. */
export type Week = {
  start: Day;
  end: Day;
};

const MIN_YEAR = 0;
const MAX_YEAR = 9999;
/** `parseISO` also accepts other ISO 8601 forms (e.g. `20261003`, or with a time): only keep the canonical ones. */
const DAY_PATTERN = /^\d{4}-\d{2}-\d{2}$/;
const MONTH_PATTERN = /^\d{4}-\d{2}$/;
const DAY_FORMAT = "uuuu-MM-dd";
const MONTH_FORMAT = "uuuu-MM";

/*
 * Days and months are handled as UTC midnights: unlike local ones, every UTC day exists and lasts 24 hours,
 * so no time zone or daylight saving time change can shift them.
 */
const UTC = { in: utc };
const WEEK = { ...UTC, weekStartsOn: 1 } as const;

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
  if (!(year >= MIN_YEAR && year <= MAX_YEAR)) {
    throw new RangeError(`Cannot format ${date} as YYYY-MM-DD`);
  }
  return format(date, DAY_FORMAT);
};

const toDay = (date: Date): Day => format(date, DAY_FORMAT, UTC);

/** @throws {RangeError} If `day` isn't an existing date formatted as `YYYY-MM-DD`. */
const parseDay = (day: Day): Date => {
  // `parseISO` rejects non-existing dates, e.g. 2026-02-30
  const date = DAY_PATTERN.test(day) ? parseISO(day, UTC) : undefined;
  if (!date || !isValid(date)) {
    throw new RangeError(`Invalid day: ${JSON.stringify(day)}`);
  }
  return date;
};

/**
 * The first day of `month`.
 *
 * @throws {RangeError} If `month` isn't formatted as `YYYY-MM`, with a month from 01 to 12.
 */
const parseMonth = (month: Month): Date => {
  const date = MONTH_PATTERN.test(month) ? parseISO(`${month}-01`, UTC) : undefined;
  if (!date || !isValid(date)) {
    throw new RangeError(`Invalid month: ${JSON.stringify(month)}`);
  }
  return date;
};

/**
 * The month of `day`.
 *
 * @throws {RangeError} If `day` is invalid.
 */
export const monthOf = (day: Day): Month => format(parseDay(day), MONTH_FORMAT, UTC);

/**
 * The first and last days of `month`.
 *
 * @throws {RangeError} If `month` is invalid.
 */
export const monthBounds = (month: Month): [first: Day, last: Day] => {
  const first = parseMonth(month);
  return [toDay(first), toDay(endOfMonth(first, UTC))];
};

/**
 * The month `count` months after `month` (before it if negative).
 *
 * @throws {RangeError} If `month` is invalid, or the result is outside 0000-9999.
 */
export const addMonths = (month: Month, count: number): Month => {
  const first = parseMonth(month);
  // date-fns would truncate a fractional `count`
  const result = Number.isInteger(count) ? addMonthsToDate(first, count, UTC) : undefined;
  const year = result && getYear(result, UTC);
  if (!result || !(year! >= MIN_YEAR && year! <= MAX_YEAR)) {
    throw new RangeError(`Cannot add ${count} months to ${month}`);
  }
  return format(result, MONTH_FORMAT, UTC);
};

/**
 * The months from `newest` back to `oldest` (both included), most recent first.
 * Empty when `oldest` is after `newest`.
 *
 * @throws {RangeError} If a month is invalid.
 */
export const monthsBetween = (newest: Month, oldest: Month): Month[] => {
  const newestDate = parseMonth(newest);
  const oldestDate = parseMonth(oldest);
  // date-fns would list reversed intervals backwards instead
  if (oldestDate > newestDate) return [];
  return eachMonthOfInterval({ start: oldestDate, end: newestDate }, UTC)
    .reverse()
    .map((date) => format(date, MONTH_FORMAT, UTC));
};

/**
 * The weeks of `month`, in chronological order. They run from Monday to Sunday, except the first and last ones,
 * which are cut at the month's bounds: every day of `month` belongs to exactly one of its weeks.
 *
 * @throws {RangeError} If `month` is invalid.
 */
export const weeksOfMonth = (month: Month): Week[] => {
  const first = parseMonth(month);
  const last = endOfMonth(first, UTC);
  return eachWeekOfInterval({ start: first, end: last }, WEEK).map((monday) => ({
    start: toDay(max([monday, first], UTC)),
    end: toDay(min([endOfWeek(monday, WEEK), last], UTC)),
  }));
};

/**
 * The week of its month containing `day`, as in `weeksOfMonth`.
 *
 * @throws {RangeError} If `day` is invalid.
 */
export const weekOf = (day: Day): Week => {
  const date = parseDay(day);
  return {
    start: toDay(max([startOfWeek(date, WEEK), startOfMonth(date, UTC)], UTC)),
    end: toDay(min([endOfWeek(date, WEEK), endOfMonth(date, UTC)], UTC)),
  };
};

/*
 * Labels use date-fns' default locale, English, like the rest of the UI.
 */

/**
 * E.g. "October 2026".
 *
 * @throws {RangeError} If `month` is invalid.
 */
export const formatMonth = (month: Month): string => format(parseMonth(month), "MMMM y", UTC);

/**
 * E.g. "Oct 5 – 11", "Sep 28 – Oct 4", or "Oct 31" for a single day.
 *
 * @throws {RangeError} If a day of `week` is invalid.
 */
export const formatWeek = ({ start, end }: Week): string => {
  const startDate = parseDay(start);
  const endDate = parseDay(end);
  const formatBoth = (startFormat: string, endFormat: string) =>
    `${format(startDate, startFormat, UTC)} – ${format(endDate, endFormat, UTC)}`;
  if (isSameDay(startDate, endDate, UTC)) return format(startDate, "MMM d", UTC);
  if (isSameMonth(startDate, endDate, UTC)) return formatBoth("MMM d", "d");
  if (isSameYear(startDate, endDate, UTC)) return formatBoth("MMM d", "MMM d");
  return formatBoth("MMM d, y", "MMM d, y");
};

/**
 * E.g. "Mon 5".
 *
 * @throws {RangeError} If `day` is invalid.
 */
export const formatDay = (day: Day): string => format(parseDay(day), "EEE d", UTC);
