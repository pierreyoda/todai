import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { formatDateTime, formatFullDateTime, formatRelativeTime, formatShortDate, formatTime } from "./timestamps";

/** Runs the enclosing `describe` block with `timeZone` as the local time zone. */
const useTimeZone = (timeZone: string) => {
  let previousTimeZone: string | undefined;
  beforeAll(() => {
    previousTimeZone = process.env.TZ;
    process.env.TZ = timeZone;
  });
  afterAll(() => {
    if (previousTimeZone === undefined) {
      delete process.env.TZ;
    } else {
      process.env.TZ = previousTimeZone;
    }
  });
};

/**
 * A local date and time: `[year, monthIndex, day, hours, minutes, seconds?]`. Turned into a `Date` in the test itself,
 * with `localDateTime`, rather than in `describe` or `it.each`: they run before `useTimeZone` sets the local time zone.
 */
type LocalDateTime = [number, number, number, number, number, number?];

const localDateTime = ([year, monthIndex, day, hours, minutes, seconds = 0]: LocalDateTime): Date =>
  new Date(year, monthIndex, day, hours, minutes, seconds);

describe("formatDateTime", () => {
  useTimeZone("Europe/Paris");

  const NOW: LocalDateTime = [2026, 9, 10, 16, 0];

  it.each<[LocalDateTime, string]>([
    [[2026, 9, 10, 14, 32], "Today, 14:32"],
    [[2026, 9, 10, 0, 5], "Today, 00:05"],
    [[2026, 9, 9, 23, 59], "Yesterday, 23:59"],
    [[2026, 9, 9, 8, 47], "Yesterday, 08:47"],
    [[2026, 9, 7, 21, 15], "Oct 7, 21:15"],
    [[2026, 0, 1, 9, 0], "Jan 1, 09:00"],
    [[2025, 5, 30, 19, 40], "Jun 30, 2025, 19:40"],
  ])("formats %j as %j", (date, label) => {
    expect(formatDateTime(localDateTime(date), localDateTime(NOW))).toBe(label);
  });

  it("follows local days, not UTC ones", () => {
    // 23:30 in Paris is still the 10th, but already the 9th at 21:30 in UTC
    const date = localDateTime([2026, 9, 10, 23, 30]);
    expect(formatDateTime(date, localDateTime([2026, 9, 10, 23, 45]))).toBe("Today, 23:30");
    expect(formatDateTime(date, localDateTime([2026, 9, 11, 0, 30]))).toBe("Yesterday, 23:30");
  });
});

describe("formatFullDateTime", () => {
  useTimeZone("Europe/Paris");

  it("formats the local date and time, to the second", () => {
    expect(formatFullDateTime(localDateTime([2026, 9, 10, 14, 32, 5]))).toBe(
      "Saturday, October 10, 2026, 14:32:05",
    );
  });
});

describe("formatRelativeTime", () => {
  // Without daylight saving time changes in between, which would shift the durations by an hour
  useTimeZone("Europe/Paris");

  const NOW: LocalDateTime = [2026, 9, 10, 16, 0];

  it.each<[LocalDateTime, string]>([
    [[2026, 9, 10, 15, 59, 30], "Just now"],
    [[2026, 9, 10, 16, 0, 30], "Just now"],
    [[2026, 9, 10, 15, 55], "5 minutes ago"],
    [[2026, 9, 10, 14, 0], "2 hours ago"],
    [[2026, 9, 8, 16, 0], "2 days ago"],
    [[2026, 6, 2, 8, 12], "3 months ago"],
    [[2026, 9, 10, 18, 0], "in 2 hours"],
  ])("formats %j as %j", (date, label) => {
    expect(formatRelativeTime(localDateTime(date), localDateTime(NOW))).toBe(label);
  });
});

describe("formatShortDate", () => {
  useTimeZone("Europe/Paris");

  const NOW: LocalDateTime = [2026, 9, 10, 16, 0];

  it.each<[LocalDateTime, string]>([
    [[2026, 9, 9, 8, 47], "Oct 9"],
    [[2026, 0, 1, 0, 0], "Jan 1"],
    [[2025, 5, 30, 19, 40], "Jun 30, 2025"],
  ])("formats %j as %j", (date, label) => {
    expect(formatShortDate(localDateTime(date), localDateTime(NOW))).toBe(label);
  });
});

describe("formatTime", () => {
  useTimeZone("Europe/Paris");

  it.each<[LocalDateTime, string]>([
    [[2026, 9, 10, 14, 32, 59], "14:32"],
    [[2026, 9, 10, 0, 5], "00:05"],
    [[2026, 9, 10, 23, 59], "23:59"],
  ])("formats %j as %j, in local time", (date, label) => {
    expect(formatTime(localDateTime(date))).toBe(label);
  });
});
