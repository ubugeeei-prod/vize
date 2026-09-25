import type {
  ResizableDirection,
  ResizableEdge,
  ResizablePhysicalEdge,
  ResizableSize,
} from "./resizable-types.ts";

/** Size constraints applied to every resize. */
export interface ResizableConstraints {
  readonly minWidth: number;
  readonly maxWidth: number;
  readonly minHeight: number;
  readonly maxHeight: number;
  /** Width / height ratio to preserve, or `null` for free resizing. */
  readonly aspectRatio: number | null;
}

/** Resolve a logical edge to a physical edge for a reading direction. */
export function resolveResizableEdge(
  edge: ResizableEdge,
  dir: ResizableDirection,
): ResizablePhysicalEdge {
  if (edge === "start") return dir === "rtl" ? "e" : "w";
  if (edge === "end") return dir === "rtl" ? "w" : "e";
  return edge;
}

/** Horizontal growth sign of an edge: +1 for east, -1 for west, 0 when width is fixed. */
export function horizontalSign(edge: ResizablePhysicalEdge): -1 | 0 | 1 {
  if (edge.includes("e")) return 1;
  if (edge.includes("w")) return -1;
  return 0;
}

/** Vertical growth sign of an edge: +1 for south, -1 for north, 0 when height is fixed. */
export function verticalSign(edge: ResizablePhysicalEdge): -1 | 0 | 1 {
  if (edge.startsWith("s")) return 1;
  if (edge.startsWith("n")) return -1;
  return 0;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), Math.max(min, max));
}

/** Clamp a size to constraints, preserving the aspect ratio when one is set. */
export function constrainResizableSize(
  size: ResizableSize,
  constraints: ResizableConstraints,
  driver: "height" | "width" = "width",
): ResizableSize {
  const ratio = constraints.aspectRatio;
  if (ratio === null || !Number.isFinite(ratio) || ratio <= 0) {
    return {
      height: clamp(size.height, constraints.minHeight, constraints.maxHeight),
      width: clamp(size.width, constraints.minWidth, constraints.maxWidth),
    };
  }
  const minWidth = Math.max(constraints.minWidth, constraints.minHeight * ratio);
  const maxWidth = Math.min(constraints.maxWidth, constraints.maxHeight * ratio);
  const requested = driver === "width" ? size.width : size.height * ratio;
  const width = clamp(requested, minWidth, maxWidth);
  return { height: width / ratio, width };
}

/** Apply a pointer or keyboard delta through an edge to a starting size. */
export function resizeByDelta(
  size: ResizableSize,
  edge: ResizablePhysicalEdge,
  deltaX: number,
  deltaY: number,
  constraints: ResizableConstraints,
): ResizableSize {
  const sx = horizontalSign(edge);
  const sy = verticalSign(edge);
  const width = size.width + sx * deltaX;
  const height = size.height + sy * deltaY;
  let driver: "height" | "width" = sx === 0 ? "height" : "width";
  if (sx !== 0 && sy !== 0) {
    const widthChange = Math.abs(width - size.width) / Math.max(size.width, 1);
    const heightChange = Math.abs(height - size.height) / Math.max(size.height, 1);
    driver = heightChange > widthChange ? "height" : "width";
  }
  return constrainResizableSize({ height, width }, constraints, driver);
}

/** Compare two sizes. */
export function resizableSizeEquals(left: ResizableSize, right: ResizableSize): boolean {
  return left.width === right.width && left.height === right.height;
}

/** Resolve the `lockAspectRatio` option to a width / height ratio, or `null` when unlocked. */
export function resolveResizableAspectRatio(
  lock: boolean | number,
  basis: ResizableSize,
): number | null {
  if (typeof lock === "number") return Number.isFinite(lock) && lock > 0 ? lock : null;
  if (!lock || basis.height <= 0) return null;
  return basis.width / basis.height;
}
