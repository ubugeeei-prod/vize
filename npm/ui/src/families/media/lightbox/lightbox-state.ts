import { defaultLightboxMessages } from "./lightbox-types.ts";
import type {
  LightboxDirection,
  LightboxMessageOverrides,
  LightboxMessages,
} from "./lightbox-types.ts";

/** Clamp or wrap an index into `[0, count - 1]`; empty lists resolve to `0`. */
export function resolveLightboxIndex(index: number, count: number, loop: boolean): number {
  if (!Number.isFinite(index) || count <= 0) return 0;
  const rounded = Math.round(index);
  if (loop) return ((rounded % count) + count) % count;
  return Math.min(Math.max(rounded, 0), count - 1);
}

/** Indexes of neighbours to preload around `index`, nearest first, without duplicates. */
export function lightboxPreloadIndexes(
  index: number,
  count: number,
  distance: number,
  loop: boolean,
): readonly number[] {
  const result: number[] = [];
  const reach = Number.isFinite(distance) ? Math.max(0, Math.floor(distance)) : 0;
  for (let offset = 1; offset <= reach; offset += 1) {
    for (const candidate of [index + offset, index - offset]) {
      const inRange = candidate >= 0 && candidate < count;
      if (!inRange && !loop) continue;
      const resolved = resolveLightboxIndex(candidate, count, loop);
      if (resolved !== index && !result.includes(resolved)) result.push(resolved);
    }
  }
  return result;
}

/** Outcome of one pointer swipe. */
export type LightboxSwipe = "close" | "next" | "none" | "previous";

/** Options for {@link classifyLightboxSwipe}. */
export interface LightboxSwipeOptions {
  /** Horizontal travel in CSS pixels (end minus start). */
  readonly deltaX: number;

  /** Vertical travel in CSS pixels (end minus start). */
  readonly deltaY: number;

  /** Minimum travel along the dominant axis. */
  readonly threshold: number;

  /** Reading direction; RTL swaps horizontal meaning. */
  readonly dir: LightboxDirection;

  /** Whether a downward swipe closes the viewer. */
  readonly closeOnSwipeDown: boolean;
}

/**
 * Classify a swipe by its dominant axis. A leftward swipe advances in LTR and
 * goes back in RTL; a downward swipe closes when enabled. Upward swipes and
 * short movements do nothing.
 */
export function classifyLightboxSwipe(options: LightboxSwipeOptions): LightboxSwipe {
  const { deltaX, deltaY, threshold, dir, closeOnSwipeDown } = options;
  const horizontal = Math.abs(deltaX) > Math.abs(deltaY);
  if (horizontal) {
    if (Math.abs(deltaX) < threshold) return "none";
    const forward = dir === "rtl" ? deltaX > 0 : deltaX < 0;
    return forward ? "next" : "previous";
  }
  return closeOnSwipeDown && deltaY >= threshold ? "close" : "none";
}

/** Merge partial message overrides over the English defaults. */
export function resolveLightboxMessages(
  overrides: LightboxMessageOverrides | undefined,
): LightboxMessages {
  return { ...defaultLightboxMessages, ...overrides };
}
