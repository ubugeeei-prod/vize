/**
 * Hue, in degrees, of a vector in a screen coordinate system.
 *
 * 0 is the positive x axis (right). Y grows downward, so `atan2(dy, dx)`
 * increases clockwise: down is 90, left is 180, up is 270. The result is in
 * `[0, 360)`. The origin returns 0.
 */
export function hueFromPoint(dx: number, dy: number): number {
  if (dx === 0 && dy === 0) return 0;
  const degrees = (Math.atan2(dy, dx) * 180) / Math.PI;
  return (degrees + 360) % 360;
}

/** Point on a circle for `hue`, with 0 at the right and clockwise screen angles. */
export function pointFromHue(
  hue: number,
  radius: number,
): { readonly x: number; readonly y: number } {
  const radians = (((hue % 360) + 360) % 360) * (Math.PI / 180);
  return { x: Math.cos(radians) * radius, y: Math.sin(radians) * radius };
}

/** Move a hue by `delta` degrees and wrap into `[0, 360)`. */
export function stepHue(hue: number, delta: number): number {
  return (((hue + delta) % 360) + 360) % 360;
}
