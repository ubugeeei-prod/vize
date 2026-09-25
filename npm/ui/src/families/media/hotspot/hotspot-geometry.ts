/** A point in image space, in percent of the image box (`0`–`100` on both axes). */
export interface HotspotPoint {
  readonly x: number;
  readonly y: number;
}

/** Rectangle in image-space percent. */
export interface HotspotRectShape {
  readonly type: "rect";
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** Circle in image-space percent (radius measured on both axes). */
export interface HotspotCircleShape {
  readonly type: "circle";
  readonly cx: number;
  readonly cy: number;
  readonly r: number;
}

/** Polygon in image-space percent; at least three points. */
export interface HotspotPolygonShape {
  readonly type: "polygon";
  readonly points: readonly HotspotPoint[];
}

/** Region shape accepted by HotspotArea. */
export type HotspotShape = HotspotCircleShape | HotspotPolygonShape | HotspotRectShape;

/** Direction for arrow-key movement between markers. */
export type HotspotDirection = "down" | "left" | "right" | "up";

/** A navigable marker: stable id plus position. */
export interface HotspotNavigationItem extends HotspotPoint {
  readonly id: string;
}

/**
 * Even-odd ray casting. Points on an edge may resolve either way; callers that
 * need edge inclusion should pad the polygon.
 */
export function isPointInHotspotPolygon(
  point: HotspotPoint,
  polygon: readonly HotspotPoint[],
): boolean {
  if (polygon.length < 3) return false;
  let inside = false;
  for (let index = 0, previous = polygon.length - 1; index < polygon.length; previous = index++) {
    const current = polygon[index];
    const before = polygon[previous];
    if (current === undefined || before === undefined) continue;
    const crosses =
      current.y > point.y !== before.y > point.y &&
      point.x <
        ((before.x - current.x) * (point.y - current.y)) / (before.y - current.y) + current.x;
    if (crosses) inside = !inside;
  }
  return inside;
}

/** Whether a shape contains a point (rect and circle edges are inclusive). */
export function hotspotShapeContains(shape: HotspotShape, point: HotspotPoint): boolean {
  switch (shape.type) {
    case "rect":
      return (
        point.x >= shape.x &&
        point.x <= shape.x + shape.width &&
        point.y >= shape.y &&
        point.y <= shape.y + shape.height
      );
    case "circle":
      return (point.x - shape.cx) ** 2 + (point.y - shape.cy) ** 2 <= shape.r ** 2;
    case "polygon":
      return isPointInHotspotPolygon(point, shape.points);
  }
}

/** SVG `points` attribute for a polygon shape. */
export function hotspotPolygonPoints(points: readonly HotspotPoint[]): string {
  return points.map((point) => `${point.x},${point.y}`).join(" ");
}

const UNSAFE_HREF = /^\s*(?:javascript|vbscript|data):/i;

/**
 * Trim a region link and drop empty, control-character, and script-capable
 * (`javascript:`, `vbscript:`, `data:`) URLs. Returns `undefined` when unsafe.
 */
export function sanitizeHotspotHref(value: string): string | undefined {
  const normalized = value.trim();
  if (normalized.length === 0 || UNSAFE_HREF.test(normalized)) return undefined;
  for (let index = 0; index < normalized.length; index++) {
    const code = normalized.charCodeAt(index);
    if (code < 32 || code === 127) return undefined;
  }
  return normalized;
}

/** Clamp a coordinate into `[0, 100]`; non-finite values resolve to `0`. */
export function clampHotspotCoordinate(value: number): number {
  return Number.isFinite(value) ? Math.min(100, Math.max(0, value)) : 0;
}

/**
 * The marker spatially next to `fromId` in `direction`.
 *
 * Candidates must lie strictly ahead on the movement axis, inside a 90° cone
 * (cross-axis offset no larger than the distance ahead). The score weights
 * the cross-axis offset twice as heavily as the distance ahead, so the nearest
 * marker roughly in line wins. Ties keep input order. Returns `null` when no
 * marker lies in that direction or `fromId` is unknown.
 */
export function nextHotspotInDirection(
  items: readonly HotspotNavigationItem[],
  fromId: string,
  direction: HotspotDirection,
): string | null {
  const from = items.find((item) => item.id === fromId);
  if (from === undefined) return null;
  let best: string | null = null;
  let bestScore = Number.POSITIVE_INFINITY;
  for (const item of items) {
    if (item.id === fromId) continue;
    const dx = item.x - from.x;
    const dy = item.y - from.y;
    const ahead =
      direction === "right" ? dx : direction === "left" ? -dx : direction === "down" ? dy : -dy;
    const cross = direction === "left" || direction === "right" ? Math.abs(dy) : Math.abs(dx);
    // Only markers inside the 90° cone around the movement axis qualify.
    if (!(ahead > 0) || cross > ahead) continue;
    const score = ahead + cross * 2;
    if (score < bestScore) {
      best = item.id;
      bestScore = score;
    }
  }
  return best;
}
