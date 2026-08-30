const safeSegment = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

/** Return whether two public Stepper values represent the same current step. */
export function stepperValueEquals(left: string | null, right: string | null): boolean {
  return Object.is(left, right);
}

/** Create a deterministic, DOM-id-safe segment for an arbitrary step value. */
export function getStepperValueIdSegment(value: string): string {
  if (safeSegment.test(value)) return `value-${value}`;

  const readable = value
    .replaceAll(/[^A-Za-z0-9_-]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 32);
  return `value-${readable || "empty"}-${hashStepperValue(value)}`;
}

function hashStepperValue(value: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index++) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(36);
}
