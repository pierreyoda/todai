import type { Day, Month } from "../client/types";

/** A week of a month: Monday to Sunday, cut at the month's bounds. */
export type Week = {
  start: Day;
  end: Day;
};

const MIN_YEAR = 0;
const MAX_YEAR = 9999;
const DAY_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/;
const MONTH_PATTERN = /^(\d{4})-(\d{2})$/;
/** Labels match the rest of the UI, which is in English. */
const LOCALE = "en-US";

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
  if (!(year >= MIN_YEAR && year <= MAX_YEAR)) {
    throw new RangeError(`Cannot format ${date} as YYYY-MM-DD`);
  }
  return [
    padDatePart(year, 4),
    padDatePart(date.getMonth() + 1, 2),
    padDatePart(date.getDate(), 2),
  ].join("-");
};

/*
 * Days and months are handled as UTC midnights: unlike local dates, every UTC day lasts 24 hours,
 * so no time zone or daylight saving time change can shift them.
 */

/** UTC midnight of the given calendar date; `monthIndex` and `date` overflow into the next units. */
const utcDate = (year: number, monthIndex: number, date: number): Date => {
  const result = new Date(0);
  // Not `Date.UTC`, which maps years 0-99 to 1900-1999
  result.setUTCFullYear(year, monthIndex, date);
  return result;
};

const utcDateToDay = (date: Date): Day =>
  [
    padDatePart(date.getUTCFullYear(), 4),
    padDatePart(date.getUTCMonth() + 1, 2),
    padDatePart(date.getUTCDate(), 2),
  ].join("-");

/** @throws {RangeError} If `day` isn't an existing date formatted as `YYYY-MM-DD`. */
const parseDay = (day: Day): Date => {
  const match = DAY_PATTERN.exec(day);
  const date = match && utcDate(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
  // Also rejects overflowing dates, e.g. 2026-02-30 becoming 2026-03-02
  if (!date || utcDateToDay(date) !== day) {
    throw new RangeError(`Invalid day: ${JSON.stringify(day)}`);
  }
  return date;
};

/** @throws {RangeError} If `month` isn't formatted as `YYYY-MM`, with a month from 01 to 12. */
const parseMonth = (month: Month): { year: number; monthIndex: number } => {
  const match = MONTH_PATTERN.exec(month);
  const monthIndex = Number(match?.[2]) - 1;
  if (!match || !(monthIndex >= 0 && monthIndex <= 11)) {
    throw new RangeError(`Invalid month: ${JSON.stringify(month)}`);
  }
  return { year: Number(match[1]), monthIndex };
};

const addDays = (date: Date, count: number): Date =>
  utcDate(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate() + count);

/** 0 for Monday to 6 for Sunday. */
const mondayBasedWeekday = (date: Date): number => (date.getUTCDay() + 6) % 7;

/**
 * The month of `day`.
 *
 * @throws {RangeError} If `day` is invalid.
 */
export const monthOf = (day: Day): Month => {
  parseDay(day);
  return day.slice(0, 7);
};

/**
 * The first and last days of `month`.
 *
 * @throws {RangeError} If `month` is invalid.
 */
export const monthBounds = (month: Month): [first: Day, last: Day] => {
  const { year, monthIndex } = parseMonth(month);
  // Day 0 of the next month is the last day of this one
  const lastDate = utcDate(year, monthIndex + 1, 0).getUTCDate();
  return [`${month}-01`, `${month}-${padDatePart(lastDate, 2)}`];
};

/**
 * The month `count` months after `month` (before it if negative).
 *
 * @throws {RangeError} If `month` is invalid, or the result is outside 0000-9999.
 */
export const addMonths = (month: Month, count: number): Month => {
  const { year, monthIndex } = parseMonth(month);
  const total = year * 12 + monthIndex + count;
  const resultYear = Math.floor(total / 12);
  if (!Number.isInteger(count) || resultYear < MIN_YEAR || resultYear > MAX_YEAR) {
    throw new RangeError(`Cannot add ${count} months to ${month}`);
  }
  return `${padDatePart(resultYear, 4)}-${padDatePart((total % 12) + 1, 2)}`;
};

/**
 * The months from `newest` back to `oldest` (both included), most recent first.
 * Empty when `oldest` is after `newest`.
 *
 * @throws {RangeError} If a month is invalid.
 */
export const monthsBetween = (newest: Month, oldest: Month): Month[] => {
  parseMonth(newest);
  parseMonth(oldest);
  const months: Month[] = [];
  // `YYYY-MM` strings compare chronologically
  for (let month = newest; month >= oldest; ) {
    months.push(month);
    // Stops before going below `oldest`, which may be the minimum month
    if (month === oldest) break;
    month = addMonths(month, -1);
  }
  return months;
};

/**
 * The weeks of `month`, in chronological order. They run from Monday to Sunday, except the first and last ones,
 * which are cut at the month's bounds: every day of `month` belongs to exactly one of its weeks.
 *
 * @throws {RangeError} If `month` is invalid.
 */
export const weeksOfMonth = (month: Month): Week[] => {
  const [first, last] = monthBounds(month).map(parseDay);
  const weeks: Week[] = [];
  for (let start = first; start <= last; ) {
    const sunday = addDays(start, 6 - mondayBasedWeekday(start));
    const end = sunday < last ? sunday : last;
    weeks.push({ start: utcDateToDay(start), end: utcDateToDay(end) });
    start = addDays(end, 1);
  }
  return weeks;
};

/**
 * The week of its month containing `day`, as in `weeksOfMonth`.
 *
 * @throws {RangeError} If `day` is invalid.
 */
export const weekOf = (day: Day): Week => {
  const date = parseDay(day);
  const weekday = mondayBasedWeekday(date);
  const monday = addDays(date, -weekday);
  const sunday = addDays(date, 6 - weekday);
  const [first, last] = monthBounds(monthOf(day)).map(parseDay);
  return {
    start: utcDateToDay(monday < first ? first : monday),
    end: utcDateToDay(sunday > last ? last : sunday),
  };
};

const monthFormat = new Intl.DateTimeFormat(LOCALE, { month: "long", year: "numeric", timeZone: "UTC" });
const shortDateFormat = new Intl.DateTimeFormat(LOCALE, { month: "short", day: "numeric", timeZone: "UTC" });
const weekdayFormat = new Intl.DateTimeFormat(LOCALE, { weekday: "short", timeZone: "UTC" });

/**
 * E.g. "October 2026".
 *
 * @throws {RangeError} If `month` is invalid.
 */
export const formatMonth = (month: Month): string => {
  const { year, monthIndex } = parseMonth(month);
  return monthFormat.format(utcDate(year, monthIndex, 1));
};

/**
 * E.g. "Oct 5 – 11", or "Oct 31" for a single day.
 *
 * @throws {RangeError} If a day of `week` is invalid.
 */
export const formatWeek = ({ start, end }: Week): string =>
  shortDateFormat.formatRange(parseDay(start), parseDay(end));

/**
 * E.g. "Mon 5".
 *
 * @throws {RangeError} If `day` is invalid.
 */
export const formatDay = (day: Day): string => {
  const date = parseDay(day);
  // Built by hand: "en-US" puts the weekday after the date ("5 Mon")
  return `${weekdayFormat.format(date)} ${date.getUTCDate()}`;
};
