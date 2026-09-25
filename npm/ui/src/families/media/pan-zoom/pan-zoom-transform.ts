import type {
  PanZoomBounds,
  PanZoomPoint,
  PanZoomSize,
  PanZoomTransform,
} from "./pan-zoom-types.ts";

/** Identity transform. */
export const PAN_ZOOM_IDENTITY: PanZoomTransform = Object.freeze({ x: 0, y: 0, scale: 1 });

/** Pixels per `WheelEvent.DOM_DELTA_LINE` unit. */
export const PAN_ZOOM_LINE_HEIGHT = 16;

const EPSILON = 1e-6;

/** Scale limits applied by {@link clampScale}. */
export interface PanZoomScaleLimits {
  readonly minScale: number;
  readonly maxScale: number;
}

function finite(value: number, fallback: number): number {
  return Number.isFinite(value) ? value : fallback;
}

/** Freeze a transform, replacing non-finite members with identity values. */
export function createTransform(x: number, y: number, scale: number): PanZoomTransform {
  const safeScale = finite(scale, 1);
  return Object.freeze({
    x: finite(x, 0),
    y: finite(y, 0),
    scale: safeScale > 0 ? safeScale : 1,
  });
}

/** Whether two transforms are equal within a sub-pixel tolerance. */
export function transformEquals(left: PanZoomTransform, right: PanZoomTransform): boolean {
  return (
    Math.abs(left.x - right.x) < EPSILON &&
    Math.abs(left.y - right.y) < EPSILON &&
    Math.abs(left.scale - right.scale) < EPSILON
  );
}

/** Clamp a zoom factor into `[minScale, maxScale]`. */
export function clampScale(scale: number, limits: PanZoomScaleLimits): number {
  const min = Math.max(EPSILON, Math.min(limits.minScale, limits.maxScale));
  const max = Math.max(min, limits.maxScale);
  return Math.min(max, Math.max(min, finite(scale, 1)));
}

/**
 * Zoom to `scale` while keeping the content point under `point` fixed on screen.
 */
export function zoomAt(
  transform: PanZoomTransform,
  scale: number,
  point: PanZoomPoint,
): PanZoomTransform {
  const ratio = scale / transform.scale;
  return createTransform(
    point.x - (point.x - transform.x) * ratio,
    point.y - (point.y - transform.y) * ratio,
    scale,
  );
}

/** Translate by a viewport-pixel delta. */
export function panBy(transform: PanZoomTransform, dx: number, dy: number): PanZoomTransform {
  return createTransform(transform.x + dx, transform.y + dy, transform.scale);
}

/** Smallest scale at which the content covers the viewport on both axes. */
export function coverScale(viewport: PanZoomSize, content: PanZoomSize): number {
  if (content.width <= 0 || content.height <= 0) return 0;
  return Math.max(viewport.width / content.width, viewport.height / content.height);
}

function clampAxis(offset: number, viewport: number, scaled: number, center: boolean): number {
  const slack = viewport - scaled;
  if (slack >= 0) return center ? slack / 2 : Math.min(slack, Math.max(0, offset));
  return Math.min(0, Math.max(slack, offset));
}

/**
 * Constrain a transform to `bounds` for the given viewport and unscaled content size.
 * Unknown (zero) sizes leave the transform unchanged.
 */
export function clampToBounds(
  transform: PanZoomTransform,
  viewport: PanZoomSize,
  content: PanZoomSize,
  bounds: PanZoomBounds,
): PanZoomTransform {
  if (bounds === "none") return transform;
  const measured =
    viewport.width > 0 && viewport.height > 0 && content.width > 0 && content.height > 0;
  if (typeof bounds === "object") {
    if (viewport.width <= 0 || viewport.height <= 0) return transform;
    const { scale } = transform;
    // The content point at the viewport center must stay inside the region.
    const centerX = (viewport.width / 2 - transform.x) / scale;
    const centerY = (viewport.height / 2 - transform.y) / scale;
    const clampedX = Math.min(bounds.x + Math.max(0, bounds.width), Math.max(bounds.x, centerX));
    const clampedY = Math.min(bounds.y + Math.max(0, bounds.height), Math.max(bounds.y, centerY));
    return createTransform(
      viewport.width / 2 - clampedX * scale,
      viewport.height / 2 - clampedY * scale,
      scale,
    );
  }
  if (!measured) return transform;
  let current = transform;
  if (bounds === "cover") {
    const minimum = coverScale(viewport, content);
    if (current.scale < minimum) {
      current = zoomAt(current, minimum, { x: viewport.width / 2, y: viewport.height / 2 });
    }
  }
  return createTransform(
    clampAxis(current.x, viewport.width, content.width * current.scale, true),
    clampAxis(current.y, viewport.height, content.height * current.scale, true),
    current.scale,
  );
}

/**
 * Transform that scales the content to fit inside the viewport (minus `padding`
 * on every side) and centers it, within the scale limits.
 */
export function fitTransform(
  viewport: PanZoomSize,
  content: PanZoomSize,
  limits: PanZoomScaleLimits,
  padding = 0,
): PanZoomTransform {
  if (content.width <= 0 || content.height <= 0 || viewport.width <= 0 || viewport.height <= 0) {
    return PAN_ZOOM_IDENTITY;
  }
  const inset = Math.max(0, padding);
  const available = {
    width: Math.max(0, viewport.width - inset * 2),
    height: Math.max(0, viewport.height - inset * 2),
  };
  const scale = clampScale(
    Math.min(available.width / content.width, available.height / content.height),
    limits,
  );
  return createTransform(
    (viewport.width - content.width * scale) / 2,
    (viewport.height - content.height * scale) / 2,
    scale,
  );
}

/** Wheel input reduced to the members the helpers read. */
export interface PanZoomWheelInput {
  readonly deltaX: number;
  readonly deltaY: number;
  /** `0` pixels, `1` lines, `2` pages. */
  readonly deltaMode: number;
}

/** Convert a wheel delta to pixels. Page deltas use the viewport size. */
export function normalizeWheelDelta(input: PanZoomWheelInput, viewport: PanZoomSize): PanZoomPoint {
  if (input.deltaMode === 1) {
    return { x: input.deltaX * PAN_ZOOM_LINE_HEIGHT, y: input.deltaY * PAN_ZOOM_LINE_HEIGHT };
  }
  if (input.deltaMode === 2) {
    return { x: input.deltaX * viewport.width, y: input.deltaY * viewport.height };
  }
  return { x: input.deltaX, y: input.deltaY };
}

/**
 * Zoom factor for a vertical wheel delta in pixels. Trackpad pinches report small
 * `ctrlKey` deltas and use a higher sensitivity so gestures feel direct.
 */
export function wheelZoomFactor(deltaY: number, pinch: boolean): number {
  const sensitivity = pinch ? 0.01 : 0.002;
  return Math.exp(-finite(deltaY, 0) * sensitivity);
}

/** Two active pointers of a pinch gesture. */
export interface PanZoomPointerPair {
  readonly first: PanZoomPoint;
  readonly second: PanZoomPoint;
}

function distance(pair: PanZoomPointerPair): number {
  return Math.hypot(pair.second.x - pair.first.x, pair.second.y - pair.first.y);
}

function midpoint(pair: PanZoomPointerPair): PanZoomPoint {
  return { x: (pair.first.x + pair.second.x) / 2, y: (pair.first.y + pair.second.y) / 2 };
}

/**
 * Transform during a pinch: the scale follows the pointer distance ratio and the
 * content point under the starting midpoint follows the current midpoint.
 */
export function pinchTransform(
  start: PanZoomTransform,
  from: PanZoomPointerPair,
  to: PanZoomPointerPair,
  limits: PanZoomScaleLimits,
): PanZoomTransform {
  const startDistance = distance(from);
  const ratio = startDistance > 0 ? distance(to) / startDistance : 1;
  const scale = clampScale(start.scale * ratio, limits);
  const anchor = midpoint(from);
  const target = midpoint(to);
  const contentX = (anchor.x - start.x) / start.scale;
  const contentY = (anchor.y - start.y) / start.scale;
  return createTransform(target.x - contentX * scale, target.y - contentY * scale, scale);
}

/** CSS `transform` value for a transform (used with `transform-origin: 0 0`). */
export function transformToCss(transform: PanZoomTransform): string {
  return `translate(${transform.x}px, ${transform.y}px) scale(${transform.scale})`;
}
