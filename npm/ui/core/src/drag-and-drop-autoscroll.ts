import { toValue } from "vue";

import type { DragAutoScrollOptions } from "./drag-and-drop-controller-types.ts";
import { measureRect, readDistance } from "./drag-and-drop-internal.ts";
import type { Point } from "./drag-and-drop-internal.ts";

/** Session-scoped auto-scroller driven by pointer samples and frame callbacks. */
export interface DragAutoScroller {
  /** Record the latest pointer sample and apply one scroll step when engaged. */
  readonly update: (point: Point) => void;

  /** Stop the frame loop and forget the last pointer sample. */
  readonly stop: () => void;
}

interface ScrollableElement extends Element {
  scrollLeft: number;
  scrollTop: number;
}

function isScrollable(element: Element | null | undefined): element is ScrollableElement {
  return Boolean(
    element && "scrollTop" in element && typeof (element as Element).scrollTop === "number",
  );
}

/**
 * Create an edge-proximity auto-scroller for one controller.
 *
 * Each pointer sample applies one immediate scroll step; while a sample stays
 * inside a threshold band, an animation-frame loop keeps scrolling so a held
 * pointer near an edge continues to advance the container.
 */
export function createDragAutoScroller(
  options: DragAutoScrollOptions | undefined,
): DragAutoScroller {
  if (!options) return Object.freeze({ update: () => undefined, stop: () => undefined });
  let lastPoint: Point | null = null;
  let frame: number | null = null;

  const step = (): boolean => {
    const point = lastPoint;
    const container = toValue(options.container);
    if (!point || !isScrollable(container)) return false;
    const rect = measureRect(container, options.getRect);
    if (!rect) return false;
    const threshold = readDistance(options.threshold, "autoScroll.threshold", 48, Number.MIN_VALUE);
    const speed = readDistance(options.speed, "autoScroll.speed", 16, Number.MIN_VALUE);
    const along = (distance: number): number =>
      distance < threshold
        ? Math.ceil(((threshold - Math.max(distance, 0)) / threshold) * speed)
        : 0;
    const nearLeft = along(point.x - rect.left);
    const nearTop = along(point.y - rect.top);
    const deltaX = nearLeft > 0 ? -nearLeft : along(rect.right - point.x);
    const deltaY = nearTop > 0 ? -nearTop : along(rect.bottom - point.y);
    if (deltaX === 0 && deltaY === 0) return false;
    container.scrollLeft += deltaX;
    container.scrollTop += deltaY;
    return true;
  };

  const loop = () => {
    frame = null;
    if (!step()) return;
    const raf = globalThis.requestAnimationFrame;
    if (typeof raf === "function") frame = raf(loop);
  };

  return Object.freeze({
    update(point: Point) {
      lastPoint = point;
      const engaged = step();
      const raf = globalThis.requestAnimationFrame;
      if (engaged && frame === null && typeof raf === "function") frame = raf(loop);
    },
    stop() {
      lastPoint = null;
      if (frame !== null && typeof globalThis.cancelAnimationFrame === "function") {
        globalThis.cancelAnimationFrame(frame);
      }
      frame = null;
    },
  });
}
