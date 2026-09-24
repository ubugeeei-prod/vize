import type { CarouselDirection, CarouselOrientation } from "./carousel-types.ts";

/** Axis-aligned box read from `getBoundingClientRect()`. */
export interface CarouselRect {
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
  readonly left: number;
}

/** Scroll metrics read from the scroll container along the carousel axis. */
export interface CarouselScrollMetrics {
  /** `scrollLeft` or `scrollTop`; negative in RTL horizontal containers. */
  readonly position: number;

  /** `clientWidth` or `clientHeight`. */
  readonly clientSize: number;

  /** `scrollWidth` or `scrollHeight`. */
  readonly scrollSize: number;
}

/** Layout axis options shared by the geometry helpers. */
export interface CarouselAxis {
  readonly orientation: CarouselOrientation;
  readonly dir: CarouselDirection;
}

/** Scroll edges reached by the viewport. `null` means the edge is unknown (no overflow measured). */
export interface CarouselScrollEdges {
  readonly atStart: boolean | null;
  readonly atEnd: boolean | null;
}

const EDGE_TOLERANCE = 1;

/**
 * Clamp or wrap a requested slide index into `[0, count - 1]`.
 *
 * Non-integer requests round toward the nearest slide. An empty carousel always
 * resolves to `0` so state stays well-formed while slides mount.
 */
export function resolveSlideIndex(index: number, count: number, loop: boolean): number {
  if (!Number.isFinite(index) || count <= 0) return 0;
  const rounded = Math.round(index);
  if (loop) return ((rounded % count) + count) % count;
  return Math.min(Math.max(rounded, 0), count - 1);
}

/**
 * Distance from the viewport's start edge to a slide's start edge along the axis.
 *
 * The start edge is `left` for LTR, `right` for RTL, and `top` for vertical
 * carousels. Adding the distance to the current scroll position yields the scroll
 * position that aligns the slide with the viewport start in every direction mode.
 */
export function slideStartOffset(
  viewport: CarouselRect,
  slide: CarouselRect,
  axis: CarouselAxis,
): number {
  if (axis.orientation === "vertical") return slide.top - viewport.top;
  return axis.dir === "rtl" ? slide.right - viewport.right : slide.left - viewport.left;
}

/** Scroll position that aligns one slide with the viewport start edge. */
export function scrollPositionForSlide(
  metrics: CarouselScrollMetrics,
  viewport: CarouselRect,
  slide: CarouselRect,
  axis: CarouselAxis,
): number {
  return metrics.position + slideStartOffset(viewport, slide, axis);
}

/**
 * Index of the slide whose start edge is closest to the viewport start edge.
 * Unmeasured (`null`) slides are skipped. Returns `-1` without measured slides.
 */
export function nearestSlideIndex(
  viewport: CarouselRect,
  slides: readonly (CarouselRect | null)[],
  axis: CarouselAxis,
): number {
  let nearest = -1;
  let nearestDistance = Number.POSITIVE_INFINITY;
  slides.forEach((slide, index) => {
    if (slide === null) return;
    const distance = Math.abs(slideStartOffset(viewport, slide, axis));
    if (distance < nearestDistance) {
      nearest = index;
      nearestDistance = distance;
    }
  });
  return nearest;
}

/** Which scroll edges the viewport currently touches. */
export function scrollEdges(metrics: CarouselScrollMetrics): CarouselScrollEdges {
  const overflow = metrics.scrollSize - metrics.clientSize;
  if (!(overflow > EDGE_TOLERANCE)) return { atStart: null, atEnd: null };
  const traveled = Math.abs(metrics.position);
  return {
    atStart: traveled <= EDGE_TOLERANCE,
    atEnd: traveled >= overflow - EDGE_TOLERANCE,
  };
}

/**
 * Slide selected after a pointer drag.
 *
 * The nearest slide wins, except that a drag past `threshold` pixels that would
 * otherwise settle on the starting slide advances one slide in the drag
 * direction, so short flicks still page.
 */
export function slideIndexAfterDrag(options: {
  readonly startIndex: number;
  readonly nearestIndex: number;
  readonly delta: number;
  readonly threshold: number;
  readonly count: number;
  readonly loop: boolean;
}): number {
  const { startIndex, nearestIndex, delta, threshold, count, loop } = options;
  if (nearestIndex !== startIndex || Math.abs(delta) < threshold) {
    return resolveSlideIndex(nearestIndex < 0 ? startIndex : nearestIndex, count, false);
  }
  return resolveSlideIndex(startIndex + (delta > 0 ? 1 : -1), count, loop);
}
