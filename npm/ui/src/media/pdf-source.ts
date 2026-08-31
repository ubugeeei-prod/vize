import { normalizeMediaSource } from "./media-source.ts";

/** Options for {@link createPDFSource}. */
export interface CreatePDFSourceOptions {
  /**
   * One-based initial PDF page.
   *
   * Existing `page` parameters in the PDF fragment are replaced while other
   * fragment parameters are preserved.
   *
   * @default undefined
   */
  readonly page?: number;

  /**
   * Permits an unencrypted HTTP resource for local development.
   *
   * @default false
   */
  readonly allowInsecure?: boolean;
}

/**
 * Validates a PDF resource and applies a standards-compatible initial page.
 *
 * @throws {TypeError} When the PDF source is unsafe or malformed.
 * @throws {RangeError} When `page` is not a positive safe integer.
 */
export function createPDFSource(source: string, options: CreatePDFSourceOptions = {}): string {
  const normalized = normalizeMediaSource(source, {
    kind: "pdf",
    allowInsecure: options.allowInsecure ?? false,
  });
  if (options.page === undefined) return normalized;

  if (!Number.isSafeInteger(options.page) || options.page < 1) {
    throw new RangeError(
      `[VIZE_UI_MEDIA_INVALID_PDF_PAGE] PDF page must be a positive safe integer; received ${String(options.page)}`,
    );
  }

  const fragmentIndex = normalized.indexOf("#");
  const resource = fragmentIndex === -1 ? normalized : normalized.slice(0, fragmentIndex);
  const currentFragment = fragmentIndex === -1 ? "" : normalized.slice(fragmentIndex + 1);
  const parameters = currentFragment
    .split("&")
    .filter((parameter) => parameter.length > 0 && !parameter.toLowerCase().startsWith("page="));

  parameters.push(`page=${options.page}`);
  return `${resource}#${parameters.join("&")}`;
}
