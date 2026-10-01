import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { dateToTodaiDate } from "./dates";

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
