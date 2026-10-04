import { afterAll, beforeAll, describe, expect, it } from "vitest";

import {
  addDays,
  addMonths,
  calendarWeeksOfMonth,
  dateToTodaiDate,
  formatDay,
  formatFullDay,
  formatMonth,
  formatWeek,
  isTodaiDate,
  monthBounds,
  monthOf,
  monthsBetween,
  weekOf,
  weeksOfMonth,
  type Week,
} from "./dates";

const TODAI_DATE_FORMAT = /^\d{4}-\d{2}-\d{2}$/;

const HOUR_MS = 60 * 60 * 1000;
const DAY_MS = 24 * HOUR_MS;

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

/** Reference implementation: the `YYYY-MM-DD` date of `date` in `timeZone`, according to `Intl`. */
const intlTodaiDate = (date: Date, timeZone: string): string => {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const part = (type: Intl.DateTimeFormatPartTypes) => parts.find((p) => p.type === type)!.value;
  return `${part("year").padStart(4, "0")}-${part("month")}-${part("day")}`;
};

/** A local date with an arbitrary year: `new Date(year, …)` maps years 0-99 to 1900-1999. */
const localDateWithYear = (year: number, monthIndex: number, day: number): Date => {
  const date = new Date(2000, monthIndex, day);
  date.setFullYear(year, monthIndex, day);
  return date;
};

describe("dateToTodaiDate", () => {
  describe("formatting", () => {
    useTimeZone("Europe/Paris");

    it("formats a date as YYYY-MM-DD", () => {
      expect(dateToTodaiDate(new Date(2026, 9, 1, 14, 30))).toBe("2026-10-01");
    });

    it("zero-pads single-digit months and days", () => {
      expect(dateToTodaiDate(new Date(2026, 0, 5))).toBe("2026-01-05");
      expect(dateToTodaiDate(new Date(2026, 8, 9))).toBe("2026-09-09");
    });

    it("keeps two-digit months and days as-is", () => {
      expect(dateToTodaiDate(new Date(2026, 11, 31))).toBe("2026-12-31");
      expect(dateToTodaiDate(new Date(2026, 9, 10))).toBe("2026-10-10");
    });

    it("ignores the time of day, from the first to the last millisecond", () => {
      expect(dateToTodaiDate(new Date(2026, 9, 1, 0, 0, 0, 0))).toBe("2026-10-01");
      expect(dateToTodaiDate(new Date(2026, 9, 1, 12, 0, 0, 0))).toBe("2026-10-01");
      expect(dateToTodaiDate(new Date(2026, 9, 1, 23, 59, 59, 999))).toBe("2026-10-01");
      expect(dateToTodaiDate(new Date(2026, 9, 2, 0, 0, 0, 0))).toBe("2026-10-02");
    });

    it("rolls over months and years", () => {
      expect(dateToTodaiDate(new Date(2026, 0, 31, 23, 59, 59, 999))).toBe("2026-01-31");
      expect(dateToTodaiDate(new Date(2026, 1, 1))).toBe("2026-02-01");
      expect(dateToTodaiDate(new Date(2026, 11, 31, 23, 59, 59, 999))).toBe("2026-12-31");
      expect(dateToTodaiDate(new Date(2027, 0, 1))).toBe("2027-01-01");
    });

    it("handles leap years", () => {
      expect(dateToTodaiDate(new Date(2024, 1, 29))).toBe("2024-02-29");
      // Divisible by 400: leap year.
      expect(dateToTodaiDate(new Date(2000, 1, 29))).toBe("2000-02-29");
      // Divisible by 100 but not by 400: not a leap year, so February 29th is March 1st.
      expect(dateToTodaiDate(new Date(2100, 1, 29))).toBe("2100-03-01");
      expect(dateToTodaiDate(new Date(2026, 1, 29))).toBe("2026-03-01");
    });

    it("zero-pads years to 4 digits", () => {
      expect(dateToTodaiDate(localDateWithYear(0, 0, 1))).toBe("0000-01-01");
      expect(dateToTodaiDate(localDateWithYear(7, 6, 4))).toBe("0007-07-04");
      expect(dateToTodaiDate(localDateWithYear(99, 11, 31))).toBe("0099-12-31");
      expect(dateToTodaiDate(localDateWithYear(999, 0, 1))).toBe("0999-01-01");
    });

    it("supports the last representable year", () => {
      expect(dateToTodaiDate(new Date(9999, 11, 31, 23, 59, 59, 999))).toBe("9999-12-31");
    });

    it("does not mutate its input", () => {
      const date = new Date(2026, 9, 1, 14, 30);
      const time = date.getTime();
      dateToTodaiDate(date);
      expect(date.getTime()).toBe(time);
    });

    it("produces strictly increasing strings for consecutive days", () => {
      // Todos are queried and sorted by this string, so lexicographic order must match date order.
      let previous = dateToTodaiDate(new Date(2023, 11, 31));
      for (let date = new Date(2024, 0, 1); date.getFullYear() < 2027; date.setDate(date.getDate() + 1)) {
        const todaiDate = dateToTodaiDate(date);
        expect(todaiDate).toMatch(TODAI_DATE_FORMAT);
        expect(todaiDate > previous).toBe(true);
        previous = todaiDate;
      }
    });
  });

  describe("invalid inputs", () => {
    useTimeZone("Europe/Paris");

    it("throws on an invalid date", () => {
      expect(() => dateToTodaiDate(new Date(Number.NaN))).toThrow(RangeError);
      expect(() => dateToTodaiDate(new Date("not a date"))).toThrow(RangeError);
    });

    it("throws on years before 0000", () => {
      expect(() => dateToTodaiDate(localDateWithYear(-1, 11, 31))).toThrow(RangeError);
    });

    it("throws on years after 9999", () => {
      expect(() => dateToTodaiDate(localDateWithYear(10000, 0, 1))).toThrow(RangeError);
    });
  });

  describe("local time zone", () => {
    describe("ahead of UTC (Europe/Paris)", () => {
      useTimeZone("Europe/Paris");

      it("runs in the expected time zone", () => {
        // Guards the other tests against a runtime ignoring `process.env.TZ`.
        expect(new Date(Date.UTC(2026, 0, 1)).getTimezoneOffset()).toBe(-60);
      });

      it("uses the local date, not the UTC one, just after local midnight", () => {
        // 2026-10-01 00:30 in Paris (UTC+2) is still 2026-09-30 in UTC.
        const date = new Date(Date.UTC(2026, 8, 30, 22, 30));
        expect(date.toISOString().slice(0, 10)).toBe("2026-09-30");
        expect(dateToTodaiDate(date)).toBe("2026-10-01");
      });

      it("handles the spring-forward DST transition", () => {
        // 2026-03-29: 02:00 local jumps to 03:00.
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 2, 28, 22, 59, 59, 999)))).toBe("2026-03-28");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 2, 28, 23, 0)))).toBe("2026-03-29");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 2, 29, 21, 59, 59, 999)))).toBe("2026-03-29");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 2, 29, 22, 0)))).toBe("2026-03-30");
      });

      it("handles the fall-back DST transition", () => {
        // 2026-10-25: 03:00 local goes back to 02:00.
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 9, 24, 21, 59, 59, 999)))).toBe("2026-10-24");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 9, 24, 22, 0)))).toBe("2026-10-25");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 9, 25, 22, 59, 59, 999)))).toBe("2026-10-25");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 9, 25, 23, 0)))).toBe("2026-10-26");
      });
    });

    describe("behind UTC (America/New_York)", () => {
      useTimeZone("America/New_York");

      it("uses the local date, not the UTC one, in the evening", () => {
        // 2026-09-30 22:00 in New York (UTC-4) is already 2026-10-01 in UTC.
        const date = new Date(Date.UTC(2026, 9, 1, 2, 0));
        expect(date.toISOString().slice(0, 10)).toBe("2026-10-01");
        expect(dateToTodaiDate(date)).toBe("2026-09-30");
      });
    });

    describe("UTC+14 (Pacific/Kiritimati)", () => {
      useTimeZone("Pacific/Kiritimati");

      it("is already the next day at UTC noon", () => {
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 9, 1, 10, 0)))).toBe("2026-10-02");
      });
    });

    describe("UTC-11 (Pacific/Pago_Pago)", () => {
      useTimeZone("Pacific/Pago_Pago");

      it("is still the previous day in the UTC morning", () => {
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 9, 1, 10, 0)))).toBe("2026-09-30");
      });
    });

    describe("non-whole-hour offset (Asia/Kathmandu, UTC+5:45)", () => {
      useTimeZone("Asia/Kathmandu");

      it("switches day at local midnight", () => {
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 8, 30, 18, 14, 59, 999)))).toBe("2026-09-30");
        expect(dateToTodaiDate(new Date(Date.UTC(2026, 8, 30, 18, 15)))).toBe("2026-10-01");
      });
    });

    describe("DST transition skipping midnight (America/Sao_Paulo, 2018)", () => {
      useTimeZone("America/Sao_Paulo");

      it("keeps the day whose midnight does not exist", () => {
        // On 2018-11-04, 00:00 local jumped to 01:00: local midnight is resolved to 01:00, the same day.
        expect(dateToTodaiDate(new Date(2018, 10, 4))).toBe("2018-11-04");
        expect(dateToTodaiDate(new Date(2018, 10, 3, 23, 59, 59, 999))).toBe("2018-11-03");
      });
    });
  });

  describe.each([
    "UTC",
    "Europe/Paris",
    "America/New_York",
    "America/Los_Angeles",
    "Asia/Tokyo",
    "Asia/Kolkata",
    "Australia/Lord_Howe",
    "Pacific/Chatham",
    "Pacific/Kiritimati",
    "Pacific/Pago_Pago",
  ])("matches Intl in %s", (timeZone) => {
    useTimeZone(timeZone);

    it("for instants spread over 3 years, including DST transitions", () => {
      const start = Date.UTC(2024, 0, 1);
      const end = Date.UTC(2027, 0, 1);
      // A 7-hour step (coprime with 24) hits every hour of the day, and DST transitions.
      for (let time = start; time < end; time += 7 * HOUR_MS) {
        const date = new Date(time);
        expect(dateToTodaiDate(date), date.toISOString()).toBe(intlTodaiDate(date, timeZone));
      }
    });

    it("around every local midnight", () => {
      for (let time = Date.UTC(2026, 0, 1); time < Date.UTC(2027, 0, 1); time += DAY_MS) {
        const midnight = new Date(time);
        midnight.setHours(0, 0, 0, 0);
        for (const date of [new Date(midnight.getTime() - 1), midnight]) {
          expect(dateToTodaiDate(date), date.toISOString()).toBe(intlTodaiDate(date, timeZone));
        }
      }
    });
  });
});

/** Time zones at both ends of UTC offsets, and ones with daylight saving time changes. */
const TIME_ZONES = ["Pacific/Kiritimati", "Pacific/Pago_Pago", "Europe/Paris", "America/Sao_Paulo"];

const INVALID_DAYS = [
  "",
  "2026-10",
  "2026-1-03",
  "2026-10-3",
  "26-10-03",
  "2026/10/03",
  "2026-10-03T00:00",
  " 2026-10-03",
  "2026-00-10",
  "2026-13-01",
  "2026-10-00",
  "2026-10-32",
  "2026-02-29",
  "2026-04-31",
];

const INVALID_MONTHS = ["", "2026", "2026-1", "26-10", "2026/10", "2026-10-01", "2026-00", "2026-13", " 2026-10"];

/** Reference implementation: every day of `month`, as UTC dates. */
const daysOfMonth = (month: string): Date[] => {
  const [year, monthNumber] = month.split("-").map(Number);
  const days: Date[] = [];
  for (let date = 1; ; date++) {
    const day = new Date(0);
    day.setUTCFullYear(year, monthNumber - 1, date);
    if (day.getUTCMonth() !== monthNumber - 1) return days;
    days.push(day);
  }
};

const toDay = (date: Date) => date.toISOString().slice(0, 10);

/** Reference implementation: the days of `month` grouped by the Monday starting their week. */
const referenceWeeks = (month: string): Week[] => {
  const weeks = new Map<string, Date[]>();
  for (const day of daysOfMonth(month)) {
    const monday = new Date(day.getTime() - ((day.getUTCDay() + 6) % 7) * 24 * 60 * 60 * 1000);
    weeks.set(toDay(monday), [...(weeks.get(toDay(monday)) ?? []), day]);
  }
  return [...weeks.values()].map((days) => ({ start: toDay(days[0]), end: toDay(days.at(-1)!) }));
};

const MONTHS_2000_2030 = monthsBetween("2030-12", "2000-01");

/** Intl uses various spaces (e.g. thin ones around range dashes), depending on its version. */
const normalizeSpaces = (text: string) => text.replace(/\s/g, " ");

describe("isTodaiDate", () => {
  it.each([
    "2026-10-03",
    "2026-01-01",
    "2026-12-31",
    "2028-02-29",
    "2000-02-29",
    "0000-01-01",
    "0999-01-31",
    "9999-12-31",
  ])("accepts the day %j", (day) => {
    expect(isTodaiDate(day)).toBe(true);
  });

  it.each([
    ...INVALID_DAYS,
    // Other ISO 8601 forms accepted by `parseISO`
    "20261003",
    "2026-10-03T12:00:00Z",
    "2026-10-03 ",
    // Divisible by 100 but not by 400: not a leap year
    "2100-02-29",
  ])("rejects the invalid day %j", (day) => {
    expect(isTodaiDate(day)).toBe(false);
  });

  it("accepts the days formatted by dateToTodaiDate", () => {
    for (const date of [new Date(2026, 9, 3), new Date(2028, 1, 29, 23, 59), new Date(2026, 11, 31, 23, 59)]) {
      expect(isTodaiDate(dateToTodaiDate(date))).toBe(true);
    }
  });
});

describe("monthOf", () => {
  it("keeps the year and month of a day", () => {
    expect(monthOf("2026-10-03")).toBe("2026-10");
    expect(monthOf("2026-12-31")).toBe("2026-12");
    expect(monthOf("0999-01-31")).toBe("0999-01");
    expect(monthOf("0000-01-01")).toBe("0000-01");
  });

  it.each(INVALID_DAYS)("rejects the invalid day %j", (day) => {
    expect(() => monthOf(day)).toThrow(RangeError);
  });
});

describe("monthBounds", () => {
  it.each([
    ["2026-01", "2026-01-31"],
    ["2026-04", "2026-04-30"],
    ["2026-12", "2026-12-31"],
    ["2026-02", "2026-02-28"],
    ["2028-02", "2028-02-29"],
    // Divisible by 100 but not by 400: not a leap year
    ["2100-02", "2100-02-28"],
    ["2000-02", "2000-02-29"],
    // Years 0-99 aren't mapped to 1900-1999
    ["0004-02", "0004-02-29"],
    ["0000-02", "0000-02-29"],
    ["9999-12", "9999-12-31"],
  ])("%s ends on %s", (month, last) => {
    expect(monthBounds(month)).toEqual([`${month}-01`, last]);
  });

  it("matches every day of the month", () => {
    for (const month of MONTHS_2000_2030) {
      const days = daysOfMonth(month);
      expect(monthBounds(month)).toEqual([toDay(days[0]), toDay(days.at(-1)!)]);
    }
  });

  it.each(INVALID_MONTHS)("rejects the invalid month %j", (month) => {
    expect(() => monthBounds(month)).toThrow(RangeError);
  });

  describe.each(TIME_ZONES)("in %s", (timeZone) => {
    useTimeZone(timeZone);

    it("doesn't depend on the local time zone", () => {
      expect(monthBounds("2026-03")).toEqual(["2026-03-01", "2026-03-31"]);
      expect(monthBounds("2026-10")).toEqual(["2026-10-01", "2026-10-31"]);
    });
  });
});

describe("addMonths", () => {
  it.each([
    ["2026-10", 0, "2026-10"],
    ["2026-10", 1, "2026-11"],
    ["2026-10", -1, "2026-09"],
    ["2026-12", 1, "2027-01"],
    ["2026-01", -1, "2025-12"],
    ["2026-10", 12, "2027-10"],
    ["2026-10", -12, "2025-10"],
    ["2026-10", 27, "2029-01"],
    ["2026-10", -22, "2024-12"],
    ["0000-02", -1, "0000-01"],
    ["9999-11", 1, "9999-12"],
    ["0000-01", 9999 * 12 + 11, "9999-12"],
  ])("%s + %i months is %s", (month, count, expected) => {
    expect(addMonths(month, count)).toBe(expected);
  });

  it.each([
    ["0000-01", -1],
    ["9999-12", 1],
    ["2026-10", 0.5],
    ["2026-10", NaN],
    ["2026-10", Infinity],
  ])("rejects %s + %s months", (month, count) => {
    expect(() => addMonths(month, count)).toThrow(RangeError);
  });

  it.each(INVALID_MONTHS)("rejects the invalid month %j", (month) => {
    expect(() => addMonths(month, 1)).toThrow(RangeError);
  });
});

describe("monthsBetween", () => {
  it("lists the months from the newest to the oldest, both included", () => {
    expect(monthsBetween("2026-02", "2025-11")).toEqual(["2026-02", "2026-01", "2025-12", "2025-11"]);
  });

  it("lists a single month", () => {
    expect(monthsBetween("2026-10", "2026-10")).toEqual(["2026-10"]);
  });

  it("is empty when the oldest month is after the newest", () => {
    expect(monthsBetween("2026-09", "2026-10")).toEqual([]);
  });

  it("lists every month of long ranges", () => {
    expect(MONTHS_2000_2030).toHaveLength(31 * 12);
    expect(new Set(MONTHS_2000_2030).size).toBe(31 * 12);
    expect(MONTHS_2000_2030.at(-1)).toBe("2000-01");
  });

  it("stops at the minimum month", () => {
    expect(monthsBetween("0000-03", "0000-01")).toEqual(["0000-03", "0000-02", "0000-01"]);
  });

  it.each(INVALID_MONTHS)("rejects the invalid month %j, even for an empty result", (month) => {
    expect(() => monthsBetween(month, "2026-10")).toThrow(RangeError);
    expect(() => monthsBetween("2026-10", month)).toThrow(RangeError);
    expect(() => monthsBetween("0000-01", month)).toThrow(RangeError);
  });
});

describe("weeksOfMonth", () => {
  it("cuts the first and last weeks at the month's bounds", () => {
    // Starts on a Thursday, ends on a Saturday
    expect(weeksOfMonth("2026-10")).toEqual([
      { start: "2026-10-01", end: "2026-10-04" },
      { start: "2026-10-05", end: "2026-10-11" },
      { start: "2026-10-12", end: "2026-10-18" },
      { start: "2026-10-19", end: "2026-10-25" },
      { start: "2026-10-26", end: "2026-10-31" },
    ]);
  });

  it("has a single-day week for a month starting on a Sunday", () => {
    // Also ends on a Monday
    const weeks = weeksOfMonth("2026-11");
    expect(weeks[0]).toEqual({ start: "2026-11-01", end: "2026-11-01" });
    expect(weeks.at(-1)).toEqual({ start: "2026-11-30", end: "2026-11-30" });
    expect(weeks).toHaveLength(6);
  });

  it("has only full weeks for a 28-day February starting on a Monday", () => {
    expect(weeksOfMonth("2021-02")).toEqual([
      { start: "2021-02-01", end: "2021-02-07" },
      { start: "2021-02-08", end: "2021-02-14" },
      { start: "2021-02-15", end: "2021-02-21" },
      { start: "2021-02-22", end: "2021-02-28" },
    ]);
  });

  it("includes February 29th of leap years", () => {
    expect(weeksOfMonth("2028-02").at(-1)).toEqual({ start: "2028-02-28", end: "2028-02-29" });
  });

  it("matches the reference implementation from 2000 to 2030", () => {
    for (const month of MONTHS_2000_2030) {
      expect(weeksOfMonth(month), month).toEqual(referenceWeeks(month));
    }
  });

  it("covers every day of the month once, with Monday-to-Sunday weeks", () => {
    for (const month of MONTHS_2000_2030) {
      const weeks = weeksOfMonth(month);
      const days = weeks.flatMap(({ start, end }) =>
        daysOfMonth(month).map(toDay).filter((day) => day >= start && day <= end),
      );
      expect(days, month).toEqual(daysOfMonth(month).map(toDay));
      expect(weeks.length, month).toBeGreaterThanOrEqual(4);
      expect(weeks.length, month).toBeLessThanOrEqual(6);
      for (const [index, { start, end }] of weeks.entries()) {
        if (index > 0) expect(new Date(start).getUTCDay(), start).toBe(1);
        if (index < weeks.length - 1) expect(new Date(end).getUTCDay(), end).toBe(0);
      }
    }
  });

  it.each(INVALID_MONTHS)("rejects the invalid month %j", (month) => {
    expect(() => weeksOfMonth(month)).toThrow(RangeError);
  });

  describe.each(TIME_ZONES)("in %s", (timeZone) => {
    useTimeZone(timeZone);

    it("doesn't depend on the local time zone", () => {
      for (const month of monthsBetween("2027-12", "2025-01")) {
        expect(weeksOfMonth(month), month).toEqual(referenceWeeks(month));
      }
    });
  });
});

describe("weekOf", () => {
  it("finds the week of its month containing the day", () => {
    expect(weekOf("2026-10-03")).toEqual({ start: "2026-10-01", end: "2026-10-04" });
    expect(weekOf("2026-10-05")).toEqual({ start: "2026-10-05", end: "2026-10-11" });
    expect(weekOf("2026-10-11")).toEqual({ start: "2026-10-05", end: "2026-10-11" });
    expect(weekOf("2026-09-30")).toEqual({ start: "2026-09-28", end: "2026-09-30" });
  });

  it("matches weeksOfMonth for every day from 2000 to 2030", () => {
    for (const month of MONTHS_2000_2030) {
      for (const week of weeksOfMonth(month)) {
        for (const day of daysOfMonth(month).map(toDay)) {
          if (day >= week.start && day <= week.end) {
            expect(weekOf(day), day).toEqual(week);
          }
        }
      }
    }
  });

  it.each(INVALID_DAYS)("rejects the invalid day %j", (day) => {
    expect(() => weekOf(day)).toThrow(RangeError);
  });

  describe.each(TIME_ZONES)("in %s", (timeZone) => {
    useTimeZone(timeZone);

    it("doesn't depend on the local time zone", () => {
      // Around the 2026 daylight saving time changes in Europe
      expect(weekOf("2026-03-29")).toEqual({ start: "2026-03-23", end: "2026-03-29" });
      expect(weekOf("2026-10-25")).toEqual({ start: "2026-10-19", end: "2026-10-25" });
      expect(weekOf("2026-10-26")).toEqual({ start: "2026-10-26", end: "2026-10-31" });
    });
  });
});

describe("formatMonth", () => {
  it.each([
    ["2026-10", "October 2026"],
    ["2026-01", "January 2026"],
    ["2026-12", "December 2026"],
  ])("formats %s as %j", (month, label) => {
    expect(formatMonth(month)).toBe(label);
  });

  it.each(INVALID_MONTHS)("rejects the invalid month %j", (month) => {
    expect(() => formatMonth(month)).toThrow(RangeError);
  });
});

describe("formatWeek", () => {
  it.each([
    [{ start: "2026-10-05", end: "2026-10-11" }, "Oct 5 – 11"],
    [{ start: "2026-10-01", end: "2026-10-04" }, "Oct 1 – 4"],
    [{ start: "2026-10-31", end: "2026-10-31" }, "Oct 31"],
    [{ start: "2026-09-28", end: "2026-10-04" }, "Sep 28 – Oct 4"],
  ])("formats %j as %j", (week, label) => {
    expect(normalizeSpaces(formatWeek(week))).toBe(label);
  });

  it.each(INVALID_DAYS)("rejects the invalid day %j", (day) => {
    expect(() => formatWeek({ start: day, end: "2026-10-04" })).toThrow(RangeError);
    expect(() => formatWeek({ start: "2026-10-01", end: day })).toThrow(RangeError);
  });
});

describe("formatDay", () => {
  it.each([
    ["2026-10-05", "Mon 5"],
    ["2026-10-01", "Thu 1"],
    ["2026-11-01", "Sun 1"],
    ["2026-10-31", "Sat 31"],
  ])("formats %s as %j", (day, label) => {
    expect(formatDay(day)).toBe(label);
  });

  it.each(INVALID_DAYS)("rejects the invalid day %j", (day) => {
    expect(() => formatDay(day)).toThrow(RangeError);
  });

  describe.each(TIME_ZONES)("in %s", (timeZone) => {
    useTimeZone(timeZone);

    it("doesn't depend on the local time zone", () => {
      expect(formatDay("2026-10-05")).toBe("Mon 5");
      expect(formatMonth("2026-10")).toBe("October 2026");
      expect(normalizeSpaces(formatWeek({ start: "2026-10-26", end: "2026-10-31" }))).toBe("Oct 26 – 31");
    });
  });
});

describe("addDays", () => {
  it.each([
    ["2026-10-04", 1, "2026-10-05"],
    ["2026-10-04", -1, "2026-10-03"],
    ["2026-10-04", 0, "2026-10-04"],
    ["2026-10-31", 1, "2026-11-01"],
    ["2026-12-31", 1, "2027-01-01"],
    ["2027-01-01", -1, "2026-12-31"],
    ["2026-10-04", 7, "2026-10-11"],
    ["2026-10-04", 365, "2027-10-04"],
    ["2028-02-28", 1, "2028-02-29"],
    ["2026-02-28", 1, "2026-03-01"],
    // DST transitions in most time zones
    ["2026-03-28", 2, "2026-03-30"],
    ["2026-10-24", 2, "2026-10-26"],
    ["0000-01-01", 0, "0000-01-01"],
    ["9999-12-30", 1, "9999-12-31"],
  ])("adds to %s %j days: %s", (day, count, expected) => {
    expect(addDays(day, count)).toBe(expected);
  });

  it.each([
    ["0000-01-01", -1],
    ["9999-12-31", 1],
    ["2026-10-04", 1.5],
    ["2026-10-04", Number.NaN],
  ])("rejects adding to %s %j days", (day, count) => {
    expect(() => addDays(day, count)).toThrow(RangeError);
  });

  it.each(INVALID_DAYS)("rejects the invalid day %j", (day) => {
    expect(() => addDays(day, 1)).toThrow(RangeError);
  });

  describe.each(TIME_ZONES)("in %s", (timeZone) => {
    useTimeZone(timeZone);

    it("walks every day of 2026 one by one", () => {
      let day = "2025-12-31";
      for (const month of monthsBetween("2026-12", "2026-01").reverse()) {
        for (const expected of daysOfMonth(month).map(toDay)) {
          day = addDays(day, 1);
          expect(day).toBe(expected);
        }
      }
    });
  });
});

describe("calendarWeeksOfMonth", () => {
  it("includes the adjacent months' days in the first and last weeks", () => {
    // Starts on a Thursday, ends on a Saturday
    const weeks = calendarWeeksOfMonth("2026-10");
    expect(weeks).toHaveLength(5);
    expect(weeks[0]).toEqual([
      "2026-09-28",
      "2026-09-29",
      "2026-09-30",
      "2026-10-01",
      "2026-10-02",
      "2026-10-03",
      "2026-10-04",
    ]);
    expect(weeks.at(-1)).toEqual([
      "2026-10-26",
      "2026-10-27",
      "2026-10-28",
      "2026-10-29",
      "2026-10-30",
      "2026-10-31",
      "2026-11-01",
    ]);
  });

  it("has six weeks for a month starting on a Sunday and ending on a Monday", () => {
    const weeks = calendarWeeksOfMonth("2026-11");
    expect(weeks).toHaveLength(6);
    expect(weeks[0][6]).toBe("2026-11-01");
    expect(weeks.at(-1)![0]).toBe("2026-11-30");
  });

  it("has only the month's days for a 28-day February starting on a Monday", () => {
    expect(calendarWeeksOfMonth("2021-02").flat()).toEqual(daysOfMonth("2021-02").map(toDay));
  });

  it("matches the weeks of the month from 2000 to 2030", () => {
    for (const month of MONTHS_2000_2030) {
      const weeks = calendarWeeksOfMonth(month);
      // Consecutive days, from Monday to Sunday
      expect(weeks.every((week) => week.length === 7), month).toBe(true);
      const days = weeks.flat();
      days.slice(1).forEach((day, index) => expect(day, month).toBe(addDays(days[index], 1)));
      // Cut at the month's bounds, they're its weeks
      expect(
        weeks.map((week) => {
          const inMonth = week.filter((day) => monthOf(day) === month);
          return { start: inMonth[0], end: inMonth.at(-1) };
        }),
        month,
      ).toEqual(weeksOfMonth(month));
    }
  });

  it.each(["0000-01", "9999-12"])("rejects %s, whose weeks go outside 0000-9999", (month) => {
    expect(() => calendarWeeksOfMonth(month)).toThrow(RangeError);
  });

  it.each(["0000-02", "9999-11"])("accepts %s", (month) => {
    expect(() => calendarWeeksOfMonth(month)).not.toThrow();
  });

  it.each(INVALID_MONTHS)("rejects the invalid month %j", (month) => {
    expect(() => calendarWeeksOfMonth(month)).toThrow(RangeError);
  });
});

describe("formatFullDay", () => {
  it.each([
    ["2026-10-05", "Monday, October 5, 2026"],
    ["2026-11-01", "Sunday, November 1, 2026"],
    ["2028-02-29", "Tuesday, February 29, 2028"],
    ["0999-01-31", "Thursday, January 31, 999"],
  ])("formats %s as %j", (day, label) => {
    expect(formatFullDay(day)).toBe(label);
  });

  it.each(INVALID_DAYS)("rejects the invalid day %j", (day) => {
    expect(() => formatFullDay(day)).toThrow(RangeError);
  });
});
