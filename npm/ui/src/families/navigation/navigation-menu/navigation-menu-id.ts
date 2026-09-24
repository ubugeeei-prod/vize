const unsafeHref = /^(?:data|javascript|vbscript):/i;

/** Drop script-capable URLs and control characters from a consumer href. */
export function normalizeNavigationMenuHref(value: string): string | undefined {
  const normalized = value.trim();
  if (normalized.length === 0 || unsafeHref.test(normalized)) return undefined;
  for (let index = 0; index < normalized.length; index++) {
    const code = normalized.charCodeAt(index);
    if (code < 32 || code === 127) return undefined;
  }
  return normalized;
}

const safeSegment = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

/** Create a deterministic, DOM-id-safe segment for an arbitrary item value. */
export function getNavigationMenuIdSegment(value: string): string {
  if (safeSegment.test(value)) return `value-${value}`;
  const readable = value
    .replaceAll(/[^A-Za-z0-9_-]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 32);
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index++) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return `value-${readable || "empty"}-${hash.toString(36)}`;
}
