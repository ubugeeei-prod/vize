import { QrCodeEncodeError } from "./qr-code-error.ts";
import { characterCountBits } from "./qr-code-tables.ts";
import type {
  QrCodeKanjiEncoder,
  QrCodeMode,
  QrCodeSegment,
  QrCodeSegmentation,
  QrCodeVersion,
} from "./qr-code-types.ts";

const ALPHANUMERIC_CHARSET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
const MAX_ECI_DESIGNATOR = 999_999;

/** Options for {@link createQrCodeSegments}. */
export interface CreateQrCodeSegmentsOptions {
  /**
   * Forced single mode, or `"auto"` to follow {@link segmentation}.
   *
   * @default "auto"
   */
  readonly mode?: QrCodeMode | "auto";

  /**
   * Automatic segmentation strategy for text.
   *
   * @default "optimal"
   */
  readonly segmentation?: QrCodeSegmentation;

  /**
   * Shift_JIS mapping that enables Kanji segments.
   *
   * @default undefined
   */
  readonly kanji?: QrCodeKanjiEncoder | undefined;

  /**
   * Version whose character-count widths the optimal segmentation targets.
   * Widths change at versions 10 and 27, so the optimum can differ per range.
   *
   * @default 1
   */
  readonly version?: QrCodeVersion;
}

/** Append `length` low bits of `value`, most significant first. */
export function appendBits(target: number[], value: number, length: number): void {
  for (let index = length - 1; index >= 0; index--) target.push((value >>> index) & 1);
}

function isDigit(codePoint: number): boolean {
  return codePoint >= 0x30 && codePoint <= 0x39;
}

function alphanumericIndex(codePoint: number): number {
  return codePoint < 0x80 ? ALPHANUMERIC_CHARSET.indexOf(String.fromCharCode(codePoint)) : -1;
}

function utf8Length(codePoint: number): number {
  if (codePoint < 0x80) return 1;
  if (codePoint < 0x800) return 2;
  return codePoint < 0x10000 ? 3 : 4;
}

function freezeSegment(mode: QrCodeSegment["mode"], count: number, bits: number[]): QrCodeSegment {
  return Object.freeze({ mode, count, bits: Object.freeze(bits) });
}

/**
 * Numeric segment: digit groups of three packed into 10 bits (7 or 4 bits for a tail).
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_INVALID_MODE` for non-digits.
 */
export function createQrCodeNumericSegment(digits: string): QrCodeSegment {
  const bits: number[] = [];
  for (const character of digits) {
    if (!isDigit(character.charCodeAt(0))) {
      throw new QrCodeEncodeError("VIZE_UI_QR_INVALID_MODE", "numeric mode accepts only digits");
    }
  }
  for (let index = 0; index < digits.length; index += 3) {
    const chunk = digits.slice(index, index + 3);
    appendBits(bits, Number.parseInt(chunk, 10), chunk.length * 3 + 1);
  }
  return freezeSegment("numeric", digits.length, bits);
}

/**
 * Alphanumeric segment: character pairs packed into 11 bits (6 bits for a tail).
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_INVALID_MODE` outside `0-9A-Z $%*+-./:`.
 */
export function createQrCodeAlphanumericSegment(text: string): QrCodeSegment {
  const values: number[] = [];
  for (const character of text) {
    const value = alphanumericIndex(character.codePointAt(0) ?? 0);
    if (value < 0) {
      throw new QrCodeEncodeError(
        "VIZE_UI_QR_INVALID_MODE",
        "alphanumeric mode accepts only 0-9, A-Z, space, and $%*+-./:",
      );
    }
    values.push(value);
  }
  const bits: number[] = [];
  let index = 0;
  for (; index + 1 < values.length; index += 2) {
    appendBits(bits, (values[index] ?? 0) * 45 + (values[index + 1] ?? 0), 11);
  }
  if (index < values.length) appendBits(bits, values[index] ?? 0, 6);
  return freezeSegment("alphanumeric", values.length, bits);
}

/** Byte segment. Text is UTF-8 encoded; bytes are used as-is. */
export function createQrCodeByteSegment(data: string | Uint8Array): QrCodeSegment {
  const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data;
  const bits: number[] = [];
  for (const byte of bytes) appendBits(bits, byte, 8);
  return freezeSegment("byte", bytes.length, bits);
}

/**
 * Kanji segment: each Shift_JIS double-byte character packed into 13 bits.
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_INVALID_MODE` for characters the
 * encoder cannot map into the QR Kanji ranges.
 */
export function createQrCodeKanjiSegment(text: string, kanji: QrCodeKanjiEncoder): QrCodeSegment {
  const bits: number[] = [];
  let count = 0;
  for (const character of text) {
    const value = kanjiValue(kanji, character.codePointAt(0) ?? 0);
    if (value === undefined) {
      throw new QrCodeEncodeError(
        "VIZE_UI_QR_INVALID_MODE",
        "kanji mode accepts only Shift_JIS double-byte characters in 0x8140-0x9FFC or 0xE040-0xEBBF",
      );
    }
    appendBits(bits, value, 13);
    count++;
  }
  return freezeSegment("kanji", count, bits);
}

/**
 * ECI header segment switching the interpretation of following byte data,
 * e.g. `26` for UTF-8. Designators use 8, 16, or 24 bits by magnitude.
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_INVALID_OPTION` outside `0..999999`.
 */
export function createQrCodeEciSegment(designator: number): QrCodeSegment {
  if (!Number.isInteger(designator) || designator < 0 || designator > MAX_ECI_DESIGNATOR) {
    throw new QrCodeEncodeError("VIZE_UI_QR_INVALID_OPTION", "eci must be an integer in 0..999999");
  }
  const bits: number[] = [];
  if (designator < 1 << 7) {
    appendBits(bits, designator, 8);
  } else if (designator < 1 << 14) {
    appendBits(bits, 0b10, 2);
    appendBits(bits, designator, 14);
  } else {
    appendBits(bits, 0b110, 3);
    appendBits(bits, designator, 21);
  }
  return freezeSegment("eci", 0, bits);
}

/** 13-bit QR Kanji value for a code point, or `undefined`. */
function kanjiValue(kanji: QrCodeKanjiEncoder, codePoint: number): number | undefined {
  const code = kanji.toShiftJis(codePoint);
  if (code === undefined) return undefined;
  let offset: number;
  if (code >= 0x8140 && code <= 0x9ffc) offset = code - 0x8140;
  else if (code >= 0xe040 && code <= 0xebbf) offset = code - 0xc140;
  else return undefined;
  const low = offset & 0xff;
  return low <= 0xbc ? (offset >>> 8) * 0xc0 + low : undefined;
}

/**
 * Total bits for segments at a version, or `Infinity` when a count overflows
 * its indicator.
 */
export function qrCodeSegmentBitLength(
  segments: readonly QrCodeSegment[],
  version: QrCodeVersion,
): number {
  let total = 0;
  for (const segment of segments) {
    if (segment.mode === "eci") {
      total += 4 + segment.bits.length;
      continue;
    }
    const countBits = characterCountBits(segment.mode, version);
    if (segment.count >= 2 ** countBits) return Number.POSITIVE_INFINITY;
    total += 4 + countBits + segment.bits.length;
  }
  return total;
}

function codePointsOf(text: string): number[] {
  const result: number[] = [];
  for (const character of text) result.push(character.codePointAt(0) ?? 0);
  return result;
}

function singleMode(codePoints: readonly number[], kanji: QrCodeKanjiEncoder | undefined) {
  if (codePoints.every(isDigit)) return "numeric";
  if (codePoints.every((codePoint) => alphanumericIndex(codePoint) >= 0)) return "alphanumeric";
  if (kanji && codePoints.every((codePoint) => kanjiValue(kanji, codePoint) !== undefined)) {
    return "kanji";
  }
  return "byte";
}

function segmentFor(mode: QrCodeMode, text: string, kanji: QrCodeKanjiEncoder | undefined) {
  switch (mode) {
    case "numeric":
      return createQrCodeNumericSegment(text);
    case "alphanumeric":
      return createQrCodeAlphanumericSegment(text);
    case "byte":
      return createQrCodeByteSegment(text);
    case "kanji":
      if (kanji === undefined) {
        throw new QrCodeEncodeError(
          "VIZE_UI_QR_INVALID_MODE",
          "kanji mode requires a kanji encoder such as qrCodeKanjiEncoder",
        );
      }
      return createQrCodeKanjiSegment(text, kanji);
  }
}

const OPTIMAL_MODES: readonly QrCodeMode[] = ["byte", "alphanumeric", "numeric", "kanji"];

/**
 * Per-character modes minimizing total bits at a version (Project Nayuki's
 * dynamic program). Costs are in sixths of a bit so numeric (10/3 bits) and
 * alphanumeric (11/2 bits) characters stay integral; partial bits round up
 * when a segment ends, which makes the fractional accounting exact.
 */
function optimalCharacterModes(
  codePoints: readonly number[],
  version: QrCodeVersion,
  kanji: QrCodeKanjiEncoder | undefined,
): QrCodeMode[] {
  const count = OPTIMAL_MODES.length;
  const headCosts = OPTIMAL_MODES.map((mode) => (4 + characterCountBits(mode, version)) * 6);
  const transitions: (QrCodeMode | null)[][] = [];
  let previous = headCosts.slice();
  for (const codePoint of codePoints) {
    const costs = Array.from<number>({ length: count }).fill(Number.POSITIVE_INFINITY);
    const from = Array.from<QrCodeMode | null>({ length: count }).fill(null);
    costs[0] = (previous[0] ?? 0) + utf8Length(codePoint) * 8 * 6;
    from[0] = "byte";
    if (alphanumericIndex(codePoint) >= 0) {
      costs[1] = (previous[1] ?? 0) + 33;
      from[1] = "alphanumeric";
    }
    if (isDigit(codePoint)) {
      costs[2] = (previous[2] ?? 0) + 20;
      from[2] = "numeric";
    }
    if (kanji && kanjiValue(kanji, codePoint) !== undefined) {
      costs[3] = (previous[3] ?? 0) + 78;
      from[3] = "kanji";
    }
    // Ending a segment after this character and starting another in mode `to`.
    const extended = costs.slice();
    const representable = from.map((entry) => entry !== null);
    for (let to = 0; to < count; to++) {
      for (let fromIndex = 0; fromIndex < count; fromIndex++) {
        if (representable[fromIndex] !== true) continue;
        const switched = Math.ceil((extended[fromIndex] ?? 0) / 6) * 6 + (headCosts[to] ?? 0);
        if (from[to] === null || switched < (costs[to] ?? 0)) {
          costs[to] = switched;
          from[to] = OPTIMAL_MODES[fromIndex] ?? "byte";
        }
      }
    }
    transitions.push(from);
    previous = costs;
  }
  let mode: QrCodeMode = "byte";
  let lowest = Number.POSITIVE_INFINITY;
  previous.forEach((cost, index) => {
    if (cost < lowest) {
      lowest = cost;
      mode = OPTIMAL_MODES[index] ?? "byte";
    }
  });
  const result: QrCodeMode[] = Array.from<QrCodeMode>({ length: codePoints.length });
  for (let index = codePoints.length - 1; index >= 0; index--) {
    const next: QrCodeMode | null | undefined = transitions[index]?.[OPTIMAL_MODES.indexOf(mode)];
    mode = next ?? "byte";
    result[index] = mode;
  }
  return result;
}

/**
 * Split text into encoded segments.
 *
 * `mode` forces one segment in that mode. Otherwise `segmentation: "single"`
 * picks the most compact single mode (numeric, alphanumeric, kanji, then
 * UTF-8 bytes) and `"optimal"` mixes modes to minimize the bit length at
 * `version`. Byte values always produce one byte segment.
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_INVALID_MODE` when a forced mode
 * cannot represent the value.
 */
export function createQrCodeSegments(
  value: string | Uint8Array,
  options: CreateQrCodeSegmentsOptions = {},
): readonly QrCodeSegment[] {
  const mode = options.mode ?? "auto";
  if (typeof value !== "string") {
    if (mode !== "auto" && mode !== "byte") {
      throw new QrCodeEncodeError(
        "VIZE_UI_QR_INVALID_MODE",
        `binary values can only use byte mode, not ${mode}`,
      );
    }
    return Object.freeze(value.length === 0 ? [] : [createQrCodeByteSegment(value)]);
  }
  if (value.length === 0) return Object.freeze([]);
  if (mode !== "auto") return Object.freeze([segmentFor(mode, value, options.kanji)]);
  const codePoints = codePointsOf(value);
  if ((options.segmentation ?? "optimal") === "single") {
    return Object.freeze([segmentFor(singleMode(codePoints, options.kanji), value, options.kanji)]);
  }
  const modes = optimalCharacterModes(codePoints, options.version ?? 1, options.kanji);
  const segments: QrCodeSegment[] = [];
  let start = 0;
  const characters = Array.from(value);
  for (let index = 1; index <= characters.length; index++) {
    if (index < characters.length && modes[index] === modes[start]) continue;
    const text = characters.slice(start, index).join("");
    segments.push(segmentFor(modes[start] ?? "byte", text, options.kanji));
    start = index;
  }
  return Object.freeze(segments);
}
