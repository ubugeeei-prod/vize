import type {
  FileUploadBuiltInRejectionCode,
  FileUploadError,
  FileUploadMessageDetail,
  FileUploadMessages,
  FileUploadRejection,
  FileUploadRejectionCode,
  FileUploadSizeStandard,
  FileUploadValidationResult,
  FileUploadValidator,
} from "./file-upload-types.ts";

/** One parsed token of a native `accept` list. */
export type FileUploadAcceptToken =
  | { readonly kind: "any" }
  | { readonly kind: "extension"; readonly extension: string }
  | { readonly kind: "mime"; readonly type: string; readonly subtype: string };

/** Whether an item is known to match an `accept` list, or cannot be decided from its type. */
export type FileUploadAcceptMatch = "accept" | "reject" | "unknown";

/** Name and type facts used for accept matching. */
export interface FileUploadAcceptCandidate {
  /** File name, used for `.ext` tokens. */
  readonly name: string;

  /** MIME type, possibly empty. */
  readonly type: string;
}

const MIME_PART = /^[a-z0-9!#$&^_.+-]+$/;
const REJECTION_CODES: readonly FileUploadRejectionCode[] = [
  "custom",
  "file-invalid-type",
  "file-too-large",
  "file-too-small",
  "too-many-files",
];

/**
 * Parse a native `accept` attribute into normalized tokens.
 *
 * Tokens are comma separated and case-insensitive. Invalid tokens are ignored exactly as the
 * browser ignores them; an empty or fully invalid list accepts every file.
 */
export function parseAccept(accept: string | null | undefined): readonly FileUploadAcceptToken[] {
  if (accept === null || accept === undefined) return [];
  const tokens: FileUploadAcceptToken[] = [];
  for (const raw of accept.split(",")) {
    const token = raw.trim().toLowerCase();
    if (token.length === 0) continue;
    if (token === "*" || token === "*/*") {
      tokens.push({ kind: "any" });
      continue;
    }
    if (token.startsWith(".")) {
      if (token.length > 1) tokens.push({ kind: "extension", extension: token });
      continue;
    }
    const essence = token.split(";")[0]?.trim() ?? "";
    const [type = "", subtype = "", ...rest] = essence.split("/");
    if (rest.length > 0 || !MIME_PART.test(type)) continue;
    if (subtype !== "*" && !MIME_PART.test(subtype)) continue;
    tokens.push({ kind: "mime", type, subtype });
  }
  return tokens;
}

function mimeEssence(value: string): readonly [string, string] | null {
  const essence = value.split(";")[0]?.trim().toLowerCase() ?? "";
  const [type = "", subtype = "", ...rest] = essence.split("/");
  if (rest.length > 0 || type.length === 0 || subtype.length === 0) return null;
  return [type, subtype];
}

function matchesMime(token: FileUploadAcceptToken, mime: readonly [string, string] | null) {
  if (token.kind !== "mime" || mime === null) return false;
  return token.type === mime[0] && (token.subtype === "*" || token.subtype === mime[1]);
}

/** Whether a file satisfies a native `accept` list (empty lists accept everything). */
export function matchesAccept(
  file: FileUploadAcceptCandidate,
  accept: string | readonly FileUploadAcceptToken[] | null | undefined,
): boolean {
  const tokens = typeof accept === "string" || accept == null ? parseAccept(accept) : accept;
  if (tokens.length === 0) return true;
  const name = file.name.toLowerCase();
  const mime = mimeEssence(file.type);
  return tokens.some(
    (token) =>
      token.kind === "any" ||
      (token.kind === "extension" && name.endsWith(token.extension)) ||
      matchesMime(token, mime),
  );
}

/**
 * Decide an accept match from a MIME type alone, as available for dragged items before drop.
 *
 * Returns `unknown` when only extension tokens could decide, or when the type is empty.
 */
export function matchAcceptType(
  type: string,
  accept: string | readonly FileUploadAcceptToken[] | null | undefined,
): FileUploadAcceptMatch {
  const tokens = typeof accept === "string" || accept == null ? parseAccept(accept) : accept;
  if (tokens.length === 0) return "accept";
  const mime = mimeEssence(type);
  if (tokens.some((token) => token.kind === "any" || matchesMime(token, mime))) return "accept";
  if (mime === null || tokens.some((token) => token.kind === "extension")) return "unknown";
  return "reject";
}

const SI_UNITS = ["byte", "kilobyte", "megabyte", "gigabyte", "terabyte", "petabyte"] as const;
const IEC_SUFFIXES = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"] as const;

/** Options for {@link formatFileSize}. */
export interface FormatFileSizeOptions {
  /**
   * BCP 47 locale used by `Intl.NumberFormat`.
   *
   * @default "en-US"
   */
  readonly locale?: string | undefined;

  /**
   * `si` uses powers of 1000 with localized units; `iec` uses powers of 1024 with KiB-style suffixes.
   *
   * @default "si"
   */
  readonly standard?: FileUploadSizeStandard | undefined;

  /**
   * Maximum fraction digits for non-byte units.
   *
   * @default 1
   */
  readonly maximumFractionDigits?: number | undefined;
}

/** Format a byte count for humans, e.g. `1.5 MB` (SI) or `1.4 MiB` (IEC). */
export function formatFileSize(bytes: number, options: FormatFileSizeOptions = {}): string {
  const locale = options.locale ?? "en-US";
  const standard = options.standard ?? "si";
  const base = standard === "si" ? 1000 : 1024;
  const safeBytes = Number.isFinite(bytes) && bytes > 0 ? bytes : 0;
  let exponent = 0;
  let value = safeBytes;
  while (value >= base && exponent < SI_UNITS.length - 1) {
    value /= base;
    exponent += 1;
  }
  const fractionDigits = exponent === 0 ? 0 : (options.maximumFractionDigits ?? 1);
  if (standard === "si") {
    return new Intl.NumberFormat(locale, {
      style: "unit",
      unit: SI_UNITS[exponent] ?? "byte",
      unitDisplay: exponent === 0 ? "long" : "short",
      maximumFractionDigits: fractionDigits,
    }).format(value);
  }
  const number = new Intl.NumberFormat(locale, {
    maximumFractionDigits: fractionDigits,
  }).format(value);
  return `${number} ${IEC_SUFFIXES[exponent] ?? "B"}`;
}

const DEFAULT_MESSAGES: {
  readonly [Code in FileUploadBuiltInRejectionCode]: (detail: FileUploadMessageDetail) => string;
} = {
  "file-invalid-type": ({ accept }) => `File type must be one of: ${accept ?? "any"}`,
  "file-too-large": ({ formatSize, maxSize }) => `File is larger than ${formatSize(maxSize ?? 0)}`,
  "file-too-small": ({ formatSize, minSize }) => `File is smaller than ${formatSize(minSize ?? 0)}`,
  "too-many-files": ({ maxFiles }) =>
    maxFiles === 1 ? "Only one file can be added" : `No more than ${maxFiles} files can be added`,
};

function isRejectionCode(value: string): value is FileUploadRejectionCode {
  return REJECTION_CODES.some((code) => code === value);
}

/** Options for {@link validateFiles}. */
export interface ValidateFilesOptions {
  /** Candidate files in arrival order. */
  readonly candidates: readonly File[];

  /** Files already held by the upload. */
  readonly current: readonly File[];

  /**
   * Native `accept` list.
   *
   * @default undefined
   */
  readonly accept?: string | undefined;

  /**
   * Whether more than one file is allowed; single uploads replace the current file.
   *
   * @default false
   */
  readonly multiple?: boolean | undefined;

  /**
   * Maximum held files when `multiple` is set.
   *
   * @default Infinity
   */
  readonly maxFiles?: number | undefined;

  /**
   * Maximum byte size per file.
   *
   * @default undefined
   */
  readonly maxSize?: number | undefined;

  /**
   * Minimum byte size per file.
   *
   * @default undefined
   */
  readonly minSize?: number | undefined;

  /**
   * Custom validation hook.
   *
   * @default undefined
   */
  readonly validate?: FileUploadValidator | undefined;

  /**
   * Replacement message factories.
   *
   * @default undefined
   */
  readonly messages?: FileUploadMessages | undefined;

  /**
   * Byte formatter for messages.
   *
   * @default formatFileSize
   */
  readonly formatSize?: ((bytes: number) => string) | undefined;
}

/** Result of {@link validateFiles}. */
export interface ValidateFilesResult {
  /** The complete next file list. */
  readonly files: readonly File[];

  /** Accepted candidates, in arrival order. */
  readonly accepted: readonly File[];

  /** Rejected candidates with every failure reason. */
  readonly rejected: readonly FileUploadRejection[];
}

/** Resolve the effective file cap from `multiple` and `maxFiles`. */
export function effectiveMaxFiles(multiple: boolean, maxFiles: number | undefined): number {
  if (!multiple) return 1;
  if (maxFiles === undefined || Number.isNaN(maxFiles)) return Number.POSITIVE_INFINITY;
  return Math.max(0, Math.floor(maxFiles));
}

/**
 * Validate candidates against type, size, custom, and count rules.
 *
 * Per-file checks run first so every failure is reported. Count limits then keep the earliest
 * valid candidates. A single upload replaces its current file with one valid candidate, and
 * rejects every candidate as `too-many-files` when more than one arrives at once.
 */
export function validateFiles(options: ValidateFilesOptions): ValidateFilesResult {
  const multiple = options.multiple ?? false;
  const maxFiles = effectiveMaxFiles(multiple, options.maxFiles);
  const formatSize = options.formatSize ?? ((bytes: number) => formatFileSize(bytes));
  const tokens = parseAccept(options.accept);
  const accepted: File[] = [];
  const rejected: FileUploadRejection[] = [];

  const message = (code: FileUploadBuiltInRejectionCode, file: File): FileUploadError => {
    const detail: FileUploadMessageDetail = {
      accept: options.accept,
      file,
      formatSize,
      maxFiles,
      maxSize: options.maxSize,
      minSize: options.minSize,
    };
    const factory = options.messages?.[code] ?? DEFAULT_MESSAGES[code];
    return { code, message: factory(detail) };
  };

  const fromResult = (result: FileUploadValidationResult, file: File): FileUploadError | null => {
    if (result === null || result === undefined) return null;
    if (typeof result !== "string") return result;
    if (isRejectionCode(result)) {
      return result === "custom"
        ? { code: "custom", message: "File is invalid" }
        : message(result, file);
    }
    return { code: "custom", message: result };
  };

  const tooManyAtOnce = !multiple && options.candidates.length > 1;
  const base = multiple ? options.current : [];
  let capacity = Math.max(0, maxFiles - base.length);

  for (const file of options.candidates) {
    const errors: FileUploadError[] = [];
    if (!matchesAccept(file, tokens)) errors.push(message("file-invalid-type", file));
    if (options.maxSize !== undefined && file.size > options.maxSize) {
      errors.push(message("file-too-large", file));
    }
    if (options.minSize !== undefined && file.size < options.minSize) {
      errors.push(message("file-too-small", file));
    }
    const custom = fromResult(options.validate?.(file, [...base, ...accepted]), file);
    if (custom !== null) errors.push(custom);
    if (tooManyAtOnce || (errors.length === 0 && capacity <= 0)) {
      errors.push(message("too-many-files", file));
    }
    if (errors.length > 0) {
      rejected.push({ file, errors });
      continue;
    }
    accepted.push(file);
    capacity -= 1;
  }

  return {
    accepted,
    files: accepted.length === 0 ? options.current : [...base, ...accepted],
    rejected,
  };
}
