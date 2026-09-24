import { normalizeMediaSource } from "../../../media/media-source.ts";
import type { ImageSource } from "./image-types.ts";

/** Options for {@link resolveImageCandidates}. */
export interface ResolveImageCandidatesOptions {
  /**
   * Permit unencrypted `http:` candidates for local development.
   *
   * @default false
   */
  readonly allowInsecure?: boolean;
}

/**
 * Normalize an image source or candidate chain into safe, de-duplicated sources.
 *
 * Every candidate passes the shared media-source policy for `image` resources:
 * relative, `https:`, `blob:`, and base64 `data:image/*` sources are kept, while
 * empty, malformed, and script-capable candidates are dropped instead of being
 * rendered. Order is preserved and repeated candidates collapse to their first
 * occurrence, so a failed source is never retried by the same chain.
 */
export function resolveImageCandidates(
  source: ImageSource | null | undefined,
  options: ResolveImageCandidatesOptions = {},
): readonly string[] {
  if (source === null || source === undefined) return Object.freeze([]);
  const input: readonly string[] = typeof source === "string" ? [source] : source;
  const resolved: string[] = [];
  for (const candidate of input) {
    const normalized = normalizeCandidate(candidate, options.allowInsecure === true);
    if (normalized !== undefined && !resolved.includes(normalized)) resolved.push(normalized);
  }
  return Object.freeze(resolved);
}

function normalizeCandidate(candidate: unknown, allowInsecure: boolean): string | undefined {
  if (typeof candidate !== "string") return undefined;
  try {
    return normalizeMediaSource(candidate, { kind: "image", allowInsecure });
  } catch {
    return undefined;
  }
}
