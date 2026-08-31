const DEFAULT_ASPECT_RATIO = 1;

/** Whether a ratio can be represented by native CSS aspect-ratio. */
export function isValidAspectRatio(ratio: number | undefined): boolean {
  return ratio === undefined || (Number.isFinite(ratio) && ratio > 0);
}

/** Return a positive finite ratio, falling back to a square box when invalid. */
export function normalizeAspectRatio(ratio: number | undefined): number {
  return isValidAspectRatio(ratio) ? (ratio ?? DEFAULT_ASPECT_RATIO) : DEFAULT_ASPECT_RATIO;
}
