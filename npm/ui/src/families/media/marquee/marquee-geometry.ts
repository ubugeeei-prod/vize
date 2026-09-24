import type { MarqueeDirection, MarqueeOrientation } from "./marquee-types.ts";

/** Minimum number of copies needed for a seamless loop. */
export const MARQUEE_MIN_COPIES = 2;

/** Maximum copies rendered by automatic repetition, guarding against zero-size content. */
export const MARQUEE_MAX_COPIES = 32;

/** Axis of one direction. */
export function marqueeOrientation(direction: MarqueeDirection): MarqueeOrientation {
  return direction === "up" || direction === "down" ? "vertical" : "horizontal";
}

/**
 * Copies needed so the track always covers the viewport while it moves one
 * copy-length: `ceil(viewport / distance) + 1`, never below two. A fixed
 * `repeat` wins but is still clamped to the seamless minimum.
 */
export function marqueeCopies(
  repeat: number | "auto",
  distance: number | null,
  viewport: number | null,
): number {
  if (repeat !== "auto") {
    return Number.isFinite(repeat)
      ? Math.min(MARQUEE_MAX_COPIES, Math.max(MARQUEE_MIN_COPIES, Math.floor(repeat)))
      : MARQUEE_MIN_COPIES;
  }
  if (distance === null || viewport === null || !(distance > 0) || !(viewport >= 0)) {
    return MARQUEE_MIN_COPIES;
  }
  return Math.min(
    MARQUEE_MAX_COPIES,
    Math.max(MARQUEE_MIN_COPIES, Math.ceil(viewport / distance) + 1),
  );
}

/** Seconds for one copy-length at `speed` pixels per second, or `null` when unmeasured or still. */
export function marqueeDuration(distance: number | null, speed: number): number | null {
  if (distance === null || !(distance > 0) || !(speed > 0) || !Number.isFinite(speed)) return null;
  return Math.round((distance / speed) * 1000) / 1000;
}

/** Inline custom properties published on the root. */
export function marqueeStyle(
  distance: number | null,
  duration: number | null,
  copies: number,
): string {
  const declarations = [`--vize-ui-marquee-copies: ${copies}`];
  if (distance !== null && distance > 0) {
    declarations.push(`--vize-ui-marquee-distance: ${Math.round(distance * 100) / 100}px`);
  }
  if (duration !== null) declarations.push(`--vize-ui-marquee-duration: ${duration}s`);
  return declarations.join("; ");
}
