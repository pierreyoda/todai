import { describe, expect, it } from "vitest";

import { parseResponse } from "./client";

describe("parseResponse", () => {
  it("turns timestamps into dates", () => {
    expect(
      parseResponse({
        createdAt: "2026-10-06T12:00:00Z",
        lastOpenedAt: "2026-10-06T13:30:00.5Z",
      }),
    ).toEqual({
      createdAt: new Date("2026-10-06T12:00:00Z"),
      lastOpenedAt: new Date("2026-10-06T13:30:00.5Z"),
    });
  });

  it("turns timestamps into dates in arrays and nested objects", () => {
    expect(
      parseResponse([{ todo: { updatedAt: "2026-10-06T12:00:00Z" } }]),
    ).toEqual([{ todo: { updatedAt: new Date("2026-10-06T12:00:00Z") } }]);
  });

  it("keeps other values", () => {
    const response = {
      completedAt: null,
      day: "2026-10-06",
      format: "2026-10-06T12:00:00Z",
      count: 3,
      tagIds: ["2026-10-06T12:00:00Z"],
    };
    expect(parseResponse(response)).toEqual(response);
  });

  it("keeps primitive and empty responses", () => {
    expect(parseResponse(null)).toBeNull();
    expect(parseResponse(undefined)).toBeUndefined();
    expect(parseResponse("2026-10-06T12:00:00Z")).toBe("2026-10-06T12:00:00Z");
  });
});
