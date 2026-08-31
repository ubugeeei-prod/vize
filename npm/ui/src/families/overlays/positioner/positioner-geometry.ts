import type {
  ComputePositionInput,
  ComputePositionResult,
  Placement,
  PlacementAlign,
  PlacementSide,
  Rect,
} from "./positioner-types.ts";

const sides = new Set<PlacementSide>(["bottom", "left", "right", "top"]);
const aligns = new Set<PlacementAlign>(["center", "end", "start"]);

const oppositeSide: Readonly<Record<PlacementSide, PlacementSide>> = {
  bottom: "top",
  left: "right",
  right: "left",
  top: "bottom",
};

function isFiniteNumber(value: number): boolean {
  return Number.isFinite(value);
}

function asNumber(value: number | undefined, fallback: number): number {
  return value === undefined || !isFiniteNumber(value) ? fallback : value;
}

/** Parse a placement token into side and alignment. */
export function parsePlacement(placement: Placement): {
  readonly align: PlacementAlign;
  readonly side: PlacementSide;
} {
  const [sideToken = "bottom", alignToken] = placement.split("-");
  const side = sides.has(sideToken as PlacementSide) ? (sideToken as PlacementSide) : "bottom";
  const align = aligns.has(alignToken as PlacementAlign)
    ? (alignToken as PlacementAlign)
    : "center";
  return { align, side };
}

function formatPlacement(side: PlacementSide, align: PlacementAlign): Placement {
  return align === "center" ? side : `${side}-${align}`;
}

function overflowOf(box: Rect, viewport: Rect, padding: number) {
  return {
    bottom: box.y + box.height - (viewport.y + viewport.height - padding),
    left: viewport.x + padding - box.x,
    right: box.x + box.width - (viewport.x + viewport.width - padding),
    top: viewport.y + padding - box.y,
  };
}

function intersects(left: Rect, right: Rect, padding: number): boolean {
  return (
    left.x < right.x + right.width - padding &&
    left.x + left.width > right.x + padding &&
    left.y < right.y + right.height - padding &&
    left.y + left.height > right.y + padding
  );
}

function clamp(value: number, min: number, max: number): number {
  if (max < min) return min;
  return Math.min(max, Math.max(min, value));
}

function resolveAlign(align: PlacementAlign, rtl: boolean, side: PlacementSide): PlacementAlign {
  if (align === "center" || !rtl || side === "left" || side === "right") return align;
  return align === "start" ? "end" : "start";
}

function place(input: {
  readonly align: PlacementAlign;
  readonly floating: Pick<Rect, "height" | "width">;
  readonly offset: number;
  readonly reference: Rect;
  readonly rtl: boolean;
  readonly side: PlacementSide;
}): Rect {
  const align = resolveAlign(input.align, input.rtl, input.side);
  const { floating, offset, reference, side } = input;
  let x = reference.x;
  let y = reference.y;

  if (side === "top" || side === "bottom") {
    if (align === "start") x = reference.x;
    else if (align === "end") x = reference.x + reference.width - floating.width;
    else x = reference.x + (reference.width - floating.width) / 2;
    y =
      side === "top"
        ? reference.y - floating.height - offset
        : reference.y + reference.height + offset;
  } else {
    if (align === "start") y = reference.y;
    else if (align === "end") y = reference.y + reference.height - floating.height;
    else y = reference.y + (reference.height - floating.height) / 2;
    x =
      side === "left"
        ? reference.x - floating.width - offset
        : reference.x + reference.width + offset;
  }

  return { height: floating.height, width: floating.width, x, y };
}

function mainOverflow(side: PlacementSide, overflow: ReturnType<typeof overflowOf>): number {
  return overflow[side];
}

function arrowCoords(
  box: Rect,
  reference: Rect,
  side: PlacementSide,
  arrow: Rect,
  arrowPadding: number,
): { readonly arrowX: number; readonly arrowY: number } {
  const maxX = box.width - arrow.width - arrowPadding;
  const maxY = box.height - arrow.height - arrowPadding;
  if (side === "top" || side === "bottom") {
    const center = reference.x + reference.width / 2 - box.x - arrow.width / 2;
    return {
      arrowX: clamp(center, arrowPadding, maxX),
      arrowY: side === "top" ? box.height : -arrow.height,
    };
  }
  const center = reference.y + reference.height / 2 - box.y - arrow.height / 2;
  return {
    arrowX: side === "left" ? box.width : -arrow.width,
    arrowY: clamp(center, arrowPadding, maxY),
  };
}

/**
 * Place a floating box next to a reference, flipping and shifting to stay in view.
 */
export function computePosition(input: ComputePositionInput): ComputePositionResult {
  const placement = input.placement ?? "bottom";
  const parsed = parsePlacement(placement);
  const offset = asNumber(input.offset, 0);
  const padding = asNumber(input.collisionPadding, 0);
  const arrowPadding = asNumber(input.arrowPadding, 0);
  const rtl = input.rtl === true;
  const shouldFlip = input.flip !== false;
  const shouldShift = input.shift !== false;
  const shouldHide = input.hide !== false;

  const preferred = place({
    align: parsed.align,
    floating: input.floating,
    offset,
    reference: input.reference,
    rtl,
    side: parsed.side,
  });
  const preferredOverflow = overflowOf(preferred, input.viewport, padding);

  let side = parsed.side;
  let box = preferred;
  if (shouldFlip) {
    const flippedSide = oppositeSide[parsed.side];
    const flipped = place({
      align: parsed.align,
      floating: input.floating,
      offset,
      reference: input.reference,
      rtl,
      side: flippedSide,
    });
    const flippedOverflow = overflowOf(flipped, input.viewport, padding);
    if (
      mainOverflow(parsed.side, preferredOverflow) > 0 &&
      mainOverflow(flippedSide, flippedOverflow) < mainOverflow(parsed.side, preferredOverflow)
    ) {
      side = flippedSide;
      box = flipped;
    }
  }

  if (shouldShift) {
    box = {
      ...box,
      x: clamp(
        box.x,
        input.viewport.x + padding,
        input.viewport.x + input.viewport.width - padding - box.width,
      ),
      y: clamp(
        box.y,
        input.viewport.y + padding,
        input.viewport.y + input.viewport.height - padding - box.height,
      ),
    };
  }

  const overflow = overflowOf(box, input.viewport, padding);
  const arrow = input.arrow ?? null;
  const arrowPosition =
    arrow === null
      ? { arrowX: null, arrowY: null }
      : arrowCoords(box, input.reference, side, arrow, arrowPadding);

  return {
    ...arrowPosition,
    hidden: shouldHide && !intersects(input.reference, input.viewport, padding),
    overflow,
    placement: formatPlacement(side, parsed.align),
    x: box.x,
    y: box.y,
  };
}

/** Normalize DOMRect-like measurements into a plain {@link Rect}. */
export function readRect(value: Rect | DOMRect): Rect {
  const record = value as Rect & { readonly left?: number; readonly top?: number };
  return {
    height: value.height,
    width: value.width,
    x: typeof record.x === "number" ? record.x : (record.left ?? 0),
    y: typeof record.y === "number" ? record.y : (record.top ?? 0),
  };
}
