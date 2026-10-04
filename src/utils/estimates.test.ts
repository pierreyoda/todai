import { describe, expect, it } from "vitest";

import type { Estimate } from "../client/types";
import { formatEstimate, formatEstimates } from "./estimates";

const minutes = (value: number): Estimate => ({ unit: "minutes", value });
const points = (value: number): Estimate => ({ unit: "points", value });

const INVALID_VALUES = [0, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY];

describe("formatEstimate", () => {
  it.each([
    [1, "1m"],
    [45, "45m"],
    [59, "59m"],
    [60, "1h"],
    [61, "1h 1m"],
    [90, "1h 30m"],
    [120, "2h"],
    [24 * 60, "24h"],
  ])("formats %j minutes as %j", (value, expected) => {
    expect(formatEstimate(minutes(value))).toBe(expected);
  });

  it.each([
    [1, "1 pt"],
    [2, "2 pts"],
    [13, "13 pts"],
    [100, "100 pts"],
  ])("formats %j points as %j", (value, expected) => {
    expect(formatEstimate(points(value))).toBe(expected);
  });

  it.each(INVALID_VALUES)("rejects the invalid value %j", (value) => {
    expect(() => formatEstimate(minutes(value))).toThrow(RangeError);
    expect(() => formatEstimate(points(value))).toThrow(RangeError);
  });
});

describe("formatEstimates", () => {
  it("sums each unit separately, minutes first", () => {
    expect(formatEstimates([points(3), minutes(90), points(5), minutes(45)])).toBe("2h 15m · 8 pts");
  });

  it("omits a unit without estimates", () => {
    expect(formatEstimates([minutes(30), minutes(30)])).toBe("1h");
    expect(formatEstimates([points(1)])).toBe("1 pt");
  });

  it("formats totals over a day in hours", () => {
    expect(formatEstimates([minutes(24 * 60), minutes(150)])).toBe("26h 30m");
  });

  it("skips todos without an estimate", () => {
    expect(formatEstimates([undefined, minutes(15), null, points(2)])).toBe("15m · 2 pts");
  });

  it("is undefined without any estimate", () => {
    expect(formatEstimates([])).toBeUndefined();
    expect(formatEstimates([undefined, null])).toBeUndefined();
  });

  it("accepts any iterable", () => {
    const todos = [{ estimate: minutes(20) }, { estimate: undefined }, { estimate: points(3) }];
    expect(formatEstimates(todos.map((todo) => todo.estimate).values())).toBe("20m · 3 pts");
  });

  it.each(INVALID_VALUES)("rejects the invalid value %j", (value) => {
    expect(() => formatEstimates([minutes(10), minutes(value)])).toThrow(RangeError);
    expect(() => formatEstimates([points(value)])).toThrow(RangeError);
  });
});
