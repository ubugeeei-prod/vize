/** Media resource category used to constrain inline data. */
export type MediaSourceKind = "audio" | "image" | "pdf" | "stream" | "track" | "video";

/** Options for {@link normalizeMediaSource}. */
export interface NormalizeMediaSourceOptions {
  /** Expected media resource category. */
  readonly kind: MediaSourceKind;

  /**
   * Permits an unencrypted HTTP resource for local development.
   *
   * @default false
   */
  readonly allowInsecure?: boolean;
}

const SOURCE_KINDS = new Set<MediaSourceKind>([
  "audio",
  "image",
  "pdf",
  "stream",
  "track",
  "video",
]);
const SCHEME = /^([a-z][a-z\d+.-]*):/i;
const BASE64_PAYLOAD = /^(?:[a-z\d+/]{4})*(?:[a-z\d+/]{2}==|[a-z\d+/]{3}=)?$/i;
const BINARY_MEDIA_TYPE = /^(audio|image|video)\/([a-z\d][a-z\d.+-]*)$/i;
const INVALID_PERCENT_ESCAPE = /%(?![a-f\d]{2})/i;

/**
 * Validates and normalizes a media resource reference.
 *
 * Relative references, network-relative references, encrypted remote URLs,
 * object URLs, and category-matched inline data are accepted. Unencrypted
 * remote URLs require an explicit opt-in. Unknown and script-capable schemes
 * are rejected.
 *
 * @throws {TypeError} When the resource is empty, malformed, unsafe, or does
 * not match the requested media category.
 */
export function normalizeMediaSource(source: string, options: NormalizeMediaSourceOptions): string {
  const kind = options?.kind;
  if (!SOURCE_KINDS.has(kind)) {
    throw new TypeError(`[VIZE_UI_MEDIA_INVALID_KIND] Unknown media source kind: ${String(kind)}`);
  }

  if (typeof source !== "string") {
    throw new TypeError("[VIZE_UI_MEDIA_INVALID_SOURCE] Media source must be a string");
  }

  const normalized = source.trim();
  if (normalized.length === 0 || containsControlCharacter(normalized)) {
    throw new TypeError(
      "[VIZE_UI_MEDIA_INVALID_SOURCE] Media source must be non-empty and contain no control characters",
    );
  }

  const scheme = SCHEME.exec(normalized)?.[1]?.toLowerCase();
  if (scheme === undefined || scheme === "https" || scheme === "blob") return normalized;
  if (scheme === "http" && options.allowInsecure === true) return normalized;
  if (scheme === "data" && isAllowedDataSource(normalized, kind)) return normalized;

  throw new TypeError(
    `[VIZE_UI_MEDIA_DISALLOWED_SOURCE] Source is not allowed for ${kind}: ${scheme}`,
  );
}

function isAllowedDataSource(source: string, kind: MediaSourceKind): boolean {
  if (kind === "stream") return false;

  const commaIndex = source.indexOf(",");
  if (commaIndex < 6) return false;

  const metadata = source.slice(5, commaIndex).toLowerCase();
  const payload = source.slice(commaIndex + 1);
  if (payload.length === 0) return false;

  const segments = metadata.split(";");
  const mediaType = segments.shift();
  const isBase64 = segments.at(-1) === "base64";
  if (isBase64) segments.pop();

  if (kind === "pdf") {
    return (
      mediaType === "application/pdf" && segments.length === 0 && isBase64 && isValidBase64(payload)
    );
  }

  if (kind === "track") {
    const hasValidParameters =
      segments.length === 0 || (segments.length === 1 && segments[0] === "charset=utf-8");
    if (mediaType !== "text/vtt" || !hasValidParameters) return false;
    return isBase64 ? isValidBase64(payload) : !INVALID_PERCENT_ESCAPE.test(payload);
  }

  const inlineKind = mediaType === undefined ? undefined : BINARY_MEDIA_TYPE.exec(mediaType)?.[1];
  return (
    inlineKind?.toLowerCase() === kind &&
    segments.length === 0 &&
    isBase64 &&
    isValidBase64(payload)
  );
}

function isValidBase64(payload: string): boolean {
  return payload.length > 0 && BASE64_PAYLOAD.test(payload);
}

function containsControlCharacter(value: string): boolean {
  for (const character of value) {
    const code = character.charCodeAt(0);
    if (code <= 0x1f || code === 0x7f) return true;
  }

  return false;
}
