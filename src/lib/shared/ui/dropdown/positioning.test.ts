import { describe, expect, it } from "vitest";

import { computePosition, VIEWPORT_MARGIN } from "./positioning";

const viewport = { width: 1000, height: 800 };
const floating = { width: 200, height: 300 };

/** A 100x40 anchor whose top-left corner is at (`left`, `top`). */
const anchorAt = (left: number, top: number) => ({
  left,
  top,
  right: left + 100,
  bottom: top + 40,
});

describe("computePosition", () => {
  it("places the element on the preferred side, aligned to the start", () => {
    expect(
      computePosition({
        anchor: anchorAt(100, 100),
        floating,
        viewport,
        placement: "bottom-start",
        offset: 8,
      }),
    ).toEqual({ x: 100, y: 148, side: "bottom" });
  });

  it("aligns to the end", () => {
    expect(
      computePosition({
        anchor: anchorAt(500, 100),
        floating,
        viewport,
        placement: "bottom-end",
      }),
    ).toMatchObject({ x: 400, y: 140 });
  });

  it("flips to the opposite side when the preferred one overflows", () => {
    expect(
      computePosition({
        anchor: anchorAt(100, 600),
        floating,
        viewport,
        placement: "bottom-start",
        offset: 8,
      }),
    ).toEqual({ x: 100, y: 292, side: "top" });
  });

  it("keeps the preferred side when neither side fits", () => {
    expect(
      computePosition({
        anchor: anchorAt(100, 300),
        floating: { width: 200, height: 500 },
        viewport,
        placement: "bottom-start",
      }),
    ).toMatchObject({ side: "bottom", y: 800 - 500 - VIEWPORT_MARGIN });
  });

  it("flips a submenu to the left near the right edge", () => {
    expect(
      computePosition({
        anchor: anchorAt(850, 100),
        floating,
        viewport,
        placement: "right-start",
        offset: 6,
        alignOffset: -5,
      }),
    ).toEqual({ x: 644, y: 95, side: "left" });
  });

  it("shifts the element back into the viewport", () => {
    expect(
      computePosition({
        anchor: anchorAt(900, 100),
        floating,
        viewport,
        placement: "bottom-start",
      }),
    ).toMatchObject({ x: 1000 - 200 - VIEWPORT_MARGIN });
  });

  it("keeps the margin when the element is bigger than the viewport", () => {
    expect(
      computePosition({
        anchor: anchorAt(0, 0),
        floating: { width: 2000, height: 2000 },
        viewport,
        placement: "bottom-start",
      }),
    ).toMatchObject({ x: VIEWPORT_MARGIN, y: VIEWPORT_MARGIN });
  });
});
