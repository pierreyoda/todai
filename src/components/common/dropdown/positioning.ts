export type Side = "top" | "right" | "bottom" | "left";
export type Placement = `${Side}-${"start" | "end"}`;

type Rect = Pick<DOMRect, "top" | "right" | "bottom" | "left">;
type Size = { width: number; height: number };

/** Minimum distance kept between a floating element and the viewport edges. */
export const VIEWPORT_MARGIN = 8;

const OPPOSITE_SIDE: Record<Side, Side> = {
  top: "bottom",
  right: "left",
  bottom: "top",
  left: "right",
};

const clamp = (value: number, min: number, max: number) =>
  // `min` wins when the element is bigger than the available space
  Math.max(min, Math.min(value, max));

/**
 * Viewport coordinates of a `floating` element placed next to `anchor`.
 *
 * The element is flipped to the opposite side when it overflows the viewport and the
 * opposite side has room, then shifted to stay within the viewport.
 *
 * @param offset Distance between the anchor and the element, along the placement side.
 * @param alignOffset Shift along the other axis, away from the aligned edge.
 */
export const computePosition = ({
  anchor,
  floating: { width, height },
  viewport,
  placement,
  offset = 0,
  alignOffset = 0,
}: {
  anchor: Rect;
  floating: Size;
  viewport: Size;
  placement: Placement;
  offset?: number;
  alignOffset?: number;
}): { x: number; y: number; side: Side } => {
  const [preferredSide, align] = placement.split("-") as [
    Side,
    "start" | "end",
  ];

  const mainAxis: Record<Side, number> = {
    top: anchor.top - offset - height,
    right: anchor.right + offset,
    bottom: anchor.bottom + offset,
    left: anchor.left - offset - width,
  };
  const fits: Record<Side, boolean> = {
    top: mainAxis.top >= VIEWPORT_MARGIN,
    right: mainAxis.right + width <= viewport.width - VIEWPORT_MARGIN,
    bottom: mainAxis.bottom + height <= viewport.height - VIEWPORT_MARGIN,
    left: mainAxis.left >= VIEWPORT_MARGIN,
  };
  const opposite = OPPOSITE_SIDE[preferredSide];
  const side =
    !fits[preferredSide] && fits[opposite] ? opposite : preferredSide;

  const vertical = side === "top" || side === "bottom";
  const crossAxis = vertical
    ? align === "start"
      ? anchor.left + alignOffset
      : anchor.right - width - alignOffset
    : align === "start"
      ? anchor.top + alignOffset
      : anchor.bottom - height - alignOffset;

  const x = vertical ? crossAxis : mainAxis[side];
  const y = vertical ? mainAxis[side] : crossAxis;
  return {
    x: clamp(x, VIEWPORT_MARGIN, viewport.width - width - VIEWPORT_MARGIN),
    y: clamp(y, VIEWPORT_MARGIN, viewport.height - height - VIEWPORT_MARGIN),
    side,
  };
};
