import { QrCodeEncodeError } from "./qr-code-error.ts";
import {
  appendBits,
  createQrCodeEciSegment,
  createQrCodeSegments,
  qrCodeSegmentBitLength,
} from "./qr-code-segments.ts";
import {
  QR_CODE_ECI_MODE_INDICATOR,
  QR_CODE_ERROR_CORRECTION_LEVELS,
  QR_CODE_FORMAT_BITS,
  QR_CODE_MODE_INDICATOR,
  alignmentPatternPositions,
  characterCountBits,
  dataCodewordCount,
  eccCodewordsPerBlock,
  errorCorrectionBlockCount,
  qrCodeSizeOf,
  rawDataModuleCount,
} from "./qr-code-tables.ts";
import type {
  QrCodeEncodeOptions,
  QrCodeErrorCorrection,
  QrCodeMask,
  QrCodeMatrix,
  QrCodeMode,
  QrCodeSegment,
  QrCodeValue,
  QrCodeVersion,
} from "./qr-code-types.ts";

export { QrCodeEncodeError } from "./qr-code-error.ts";

const MASKS: readonly QrCodeMask[] = [0, 1, 2, 3, 4, 5, 6, 7];
const UTF8_ECI = 26;

function isVersion(value: unknown): value is QrCodeVersion {
  return typeof value === "number" && Number.isInteger(value) && value >= 1 && value <= 40;
}

function isMask(value: unknown): value is QrCodeMask {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 7;
}

/** Multiply two elements of GF(2^8) modulo the QR polynomial `x^8 + x^4 + x^3 + x^2 + 1`. */
export function gf256Multiply(left: number, right: number): number {
  let result = 0;
  for (let bit = 7; bit >= 0; bit--) {
    result = (result << 1) ^ ((result >>> 7) * 0x11d);
    result ^= ((right >>> bit) & 1) * left;
  }
  return result & 0xff;
}

/**
 * Reed–Solomon generator polynomial coefficients for a degree, highest term
 * first with the implicit leading `1` omitted.
 */
export function reedSolomonGenerator(degree: number): readonly number[] {
  if (!Number.isInteger(degree) || degree < 1 || degree > 255) {
    throw new QrCodeEncodeError("VIZE_UI_QR_INVALID_OPTION", "degree must be in 1..255");
  }
  const result = Array.from<number>({ length: degree }).fill(0);
  result[degree - 1] = 1;
  let root = 1;
  for (let index = 0; index < degree; index++) {
    for (let term = 0; term < result.length; term++) {
      const next = term + 1 < result.length ? (result[term + 1] ?? 0) : 0;
      result[term] = gf256Multiply(result[term] ?? 0, root) ^ next;
    }
    root = gf256Multiply(root, 0x02);
  }
  return result;
}

/** Reed–Solomon error correction codewords for data under a generator. */
export function reedSolomonRemainder(
  data: readonly number[],
  generator: readonly number[],
): readonly number[] {
  const result = Array.from<number>({ length: generator.length }).fill(0);
  for (const byte of data) {
    const factor = byte ^ (result.shift() ?? 0);
    result.push(0);
    for (let index = 0; index < generator.length; index++) {
      result[index] = (result[index] ?? 0) ^ gf256Multiply(generator[index] ?? 0, factor);
    }
  }
  return result;
}

/**
 * Build the padded data codewords for segments at a version and level.
 * Exposed for conformance tests against the standard's worked examples.
 */
export function createDataCodewords(
  segments: readonly QrCodeSegment[],
  version: QrCodeVersion,
  ecc: QrCodeErrorCorrection,
): readonly number[] {
  const bits: number[] = [];
  for (const segment of segments) {
    if (segment.mode === "eci") {
      appendBits(bits, QR_CODE_ECI_MODE_INDICATOR, 4);
    } else {
      appendBits(bits, QR_CODE_MODE_INDICATOR[segment.mode], 4);
      appendBits(bits, segment.count, characterCountBits(segment.mode, version));
    }
    for (const bit of segment.bits) bits.push(bit);
  }
  const capacityBits = dataCodewordCount(version, ecc) * 8;
  appendBits(bits, 0, Math.min(4, capacityBits - bits.length));
  appendBits(bits, 0, (8 - (bits.length % 8)) % 8);
  for (let pad = 0xec; bits.length < capacityBits; pad ^= 0xec ^ 0x11) appendBits(bits, pad, 8);
  const codewords: number[] = [];
  for (let index = 0; index < bits.length; index += 8) {
    let byte = 0;
    for (let offset = 0; offset < 8; offset++) byte = (byte << 1) | (bits[index + offset] ?? 0);
    codewords.push(byte);
  }
  return codewords;
}

/** Split data into blocks, append Reed–Solomon codewords, and interleave. */
export function addErrorCorrectionAndInterleave(
  data: readonly number[],
  version: QrCodeVersion,
  ecc: QrCodeErrorCorrection,
): readonly number[] {
  const blockCount = errorCorrectionBlockCount(version, ecc);
  const blockEccLength = eccCodewordsPerBlock(version, ecc);
  const rawCodewords = Math.floor(rawDataModuleCount(version) / 8);
  const shortBlockCount = blockCount - (rawCodewords % blockCount);
  const shortBlockLength = Math.floor(rawCodewords / blockCount);
  const generator = reedSolomonGenerator(blockEccLength);
  const blocks: number[][] = [];
  let offset = 0;
  for (let index = 0; index < blockCount; index++) {
    const length = shortBlockLength - blockEccLength + (index < shortBlockCount ? 0 : 1);
    const block = data.slice(offset, offset + length);
    offset += length;
    const remainder = reedSolomonRemainder(block, generator);
    if (index < shortBlockCount) block.push(0);
    blocks.push([...block, ...remainder]);
  }
  const result: number[] = [];
  const blockLength = blocks[0]?.length ?? 0;
  for (let column = 0; column < blockLength; column++) {
    blocks.forEach((block, blockIndex) => {
      if (column !== shortBlockLength - blockEccLength || blockIndex >= shortBlockCount) {
        result.push(block[column] ?? 0);
      }
    });
  }
  return result;
}

/** 15-bit format information (level + mask with BCH code and XOR mask). */
export function formatInformationBits(ecc: QrCodeErrorCorrection, mask: QrCodeMask): number {
  const data = (QR_CODE_FORMAT_BITS[ecc] << 3) | mask;
  let remainder = data;
  for (let index = 0; index < 10; index++) {
    remainder = (remainder << 1) ^ ((remainder >>> 9) * 0x537);
  }
  return ((data << 10) | remainder) ^ 0x5412;
}

/** 18-bit version information for versions 7 and above. */
export function versionInformationBits(version: QrCodeVersion): number {
  let remainder: number = version;
  for (let index = 0; index < 12; index++) {
    remainder = (remainder << 1) ^ ((remainder >>> 11) * 0x1f25);
  }
  return (version << 12) | remainder;
}

function bitAt(value: number, index: number): boolean {
  return ((value >>> index) & 1) !== 0;
}

function maskApplies(mask: QrCodeMask, x: number, y: number): boolean {
  switch (mask) {
    case 0:
      return (x + y) % 2 === 0;
    case 1:
      return y % 2 === 0;
    case 2:
      return x % 3 === 0;
    case 3:
      return (x + y) % 3 === 0;
    case 4:
      return (Math.floor(x / 3) + Math.floor(y / 2)) % 2 === 0;
    case 5:
      return ((x * y) % 2) + ((x * y) % 3) === 0;
    case 6:
      return (((x * y) % 2) + ((x * y) % 3)) % 2 === 0;
    case 7:
      return (((x + y) % 2) + ((x * y) % 3)) % 2 === 0;
  }
}

/** Mutable module grid used while one symbol is drawn. */
class SymbolCanvas {
  readonly version: QrCodeVersion;
  readonly size: number;
  readonly modules: boolean[];
  readonly reserved: boolean[];

  constructor(version: QrCodeVersion) {
    this.version = version;
    this.size = qrCodeSizeOf(version);
    this.modules = Array.from<boolean>({ length: this.size * this.size }).fill(false);
    this.reserved = Array.from<boolean>({ length: this.size * this.size }).fill(false);
  }

  get(x: number, y: number): boolean {
    return this.modules[y * this.size + x] === true;
  }

  setFunction(x: number, y: number, dark: boolean): void {
    this.modules[y * this.size + x] = dark;
    this.reserved[y * this.size + x] = true;
  }

  drawFunctionPatterns(): void {
    const { size } = this;
    for (let index = 0; index < size; index++) {
      this.setFunction(6, index, index % 2 === 0);
      this.setFunction(index, 6, index % 2 === 0);
    }
    this.drawFinder(3, 3);
    this.drawFinder(size - 4, 3);
    this.drawFinder(3, size - 4);
    const positions = alignmentPatternPositions(this.version);
    const last = positions.length - 1;
    positions.forEach((y, row) => {
      positions.forEach((x, column) => {
        const overlapsFinder =
          (row === 0 && column === 0) ||
          (row === 0 && column === last) ||
          (row === last && column === 0);
        if (!overlapsFinder) this.drawAlignment(x, y);
      });
    });
    // Reserve format areas before data placement; real bits are drawn per mask.
    this.drawFormat(formatInformationBits("M", 0));
    this.drawVersion();
  }

  drawFinder(centerX: number, centerY: number): void {
    for (let dy = -4; dy <= 4; dy++) {
      for (let dx = -4; dx <= 4; dx++) {
        const distance = Math.max(Math.abs(dx), Math.abs(dy));
        const x = centerX + dx;
        const y = centerY + dy;
        if (x >= 0 && x < this.size && y >= 0 && y < this.size) {
          this.setFunction(x, y, distance !== 2 && distance !== 4);
        }
      }
    }
  }

  drawAlignment(centerX: number, centerY: number): void {
    for (let dy = -2; dy <= 2; dy++) {
      for (let dx = -2; dx <= 2; dx++) {
        this.setFunction(centerX + dx, centerY + dy, Math.max(Math.abs(dx), Math.abs(dy)) !== 1);
      }
    }
  }

  drawFormat(bits: number): void {
    const { size } = this;
    for (let index = 0; index <= 5; index++) this.setFunction(8, index, bitAt(bits, index));
    this.setFunction(8, 7, bitAt(bits, 6));
    this.setFunction(8, 8, bitAt(bits, 7));
    this.setFunction(7, 8, bitAt(bits, 8));
    for (let index = 9; index < 15; index++) this.setFunction(14 - index, 8, bitAt(bits, index));
    for (let index = 0; index < 8; index++)
      this.setFunction(size - 1 - index, 8, bitAt(bits, index));
    for (let index = 8; index < 15; index++)
      this.setFunction(8, size - 15 + index, bitAt(bits, index));
    this.setFunction(8, size - 8, true);
  }

  drawVersion(): void {
    if (this.version < 7) return;
    const bits = versionInformationBits(this.version);
    for (let index = 0; index < 18; index++) {
      const dark = bitAt(bits, index);
      const a = this.size - 11 + (index % 3);
      const b = Math.floor(index / 3);
      this.setFunction(a, b, dark);
      this.setFunction(b, a, dark);
    }
  }

  drawCodewords(codewords: readonly number[]): void {
    const { size } = this;
    const totalBits = codewords.length * 8;
    let bitIndex = 0;
    for (let right = size - 1; right >= 1; right -= 2) {
      if (right === 6) right = 5;
      const upward = ((right + 1) & 2) === 0;
      for (let vertical = 0; vertical < size; vertical++) {
        const y = upward ? size - 1 - vertical : vertical;
        for (let offset = 0; offset < 2; offset++) {
          const x = right - offset;
          const index = y * size + x;
          if (this.reserved[index] === true || bitIndex >= totalBits) continue;
          this.modules[index] = bitAt(codewords[bitIndex >>> 3] ?? 0, 7 - (bitIndex & 7));
          bitIndex++;
        }
      }
    }
  }

  /** XOR a mask over data modules. Applying the same mask twice restores the grid. */
  applyMask(mask: QrCodeMask): void {
    for (let y = 0; y < this.size; y++) {
      for (let x = 0; x < this.size; x++) {
        const index = y * this.size + x;
        if (this.reserved[index] !== true && maskApplies(mask, x, y)) {
          this.modules[index] = !this.modules[index];
        }
      }
    }
  }
}

/**
 * Standard mask penalty score (ISO/IEC 18004 §7.8.3): same-color runs, 2×2
 * blocks, finder-like patterns, and dark/light imbalance. Lower is better.
 */
export function qrCodePenaltyScore(size: number, modules: readonly boolean[]): number {
  // Typed scratch buffers keep this hot loop (8 masks per encode) allocation-free.
  const grid = new Uint8Array(size * size);
  let darkCount = 0;
  for (let index = 0; index < grid.length; index++) {
    if (modules[index] === true) {
      grid[index] = 1;
      darkCount++;
    }
  }
  const history = new Int32Array(7);
  let result = 0;

  for (let pass = 0; pass < 2; pass++) {
    const horizontal = pass === 0;
    for (let line = 0; line < size; line++) {
      let runDark = 0;
      let runLength = 0;
      history.fill(0);
      for (let cell = 0; cell < size; cell++) {
        const dark = grid[horizontal ? line * size + cell : cell * size + line] ?? 0;
        if (dark === runDark) {
          runLength++;
          if (runLength === 5) result += 3;
          else if (runLength > 5) result++;
        } else {
          pushFinderRun(history, runLength, size);
          if (runDark === 0) result += countFinderPatterns(history) * 40;
          runDark = dark;
          runLength = 1;
        }
      }
      if (runDark === 1) {
        pushFinderRun(history, runLength, size);
        runLength = 0;
      }
      pushFinderRun(history, runLength + size, size);
      result += countFinderPatterns(history) * 40;
    }
  }

  for (let y = 0; y < size - 1; y++) {
    const row = y * size;
    for (let x = 0; x < size - 1; x++) {
      const dark = grid[row + x];
      if (
        dark === grid[row + x + 1] &&
        dark === grid[row + size + x] &&
        dark === grid[row + size + x + 1]
      ) {
        result += 3;
      }
    }
  }

  const total = size * size;
  const k = Math.ceil(Math.abs(darkCount * 20 - total * 10) / total) - 1;
  return result + k * 10;
}

/** Shift one run length into the 7-entry finder-pattern history (light border counts as `size`). */
function pushFinderRun(history: Int32Array, runLength: number, size: number): void {
  const value = history[0] === 0 ? runLength + size : runLength;
  history.copyWithin(1, 0, 6);
  history[0] = value;
}

/** Count 1:1:3:1:1 finder-like patterns with a 4-module light margin on either side. */
function countFinderPatterns(history: Int32Array): number {
  const n = history[1] ?? 0;
  const core =
    n > 0 && history[2] === n && history[3] === n * 3 && history[4] === n && history[5] === n;
  if (!core) return 0;
  const before = history[0] ?? 0;
  const after = history[6] ?? 0;
  return (before >= n * 4 && after >= n ? 1 : 0) + (after >= n * 4 && before >= n ? 1 : 0);
}

function readVersionOption(name: string, value: unknown, fallback: QrCodeVersion): QrCodeVersion {
  if (value === undefined) return fallback;
  if (!isVersion(value)) {
    throw new QrCodeEncodeError("VIZE_UI_QR_INVALID_OPTION", `${name} must be an integer in 1..40`);
  }
  return value;
}

/** ECI designator requested by `eci` / `utf8Eci`, or `null`. */
function readEciOption(options: QrCodeEncodeOptions): number | null {
  const explicit = options.eci;
  if (options.utf8Eci === true) {
    if (explicit !== undefined && explicit !== UTF8_ECI) {
      throw new QrCodeEncodeError(
        "VIZE_UI_QR_INVALID_OPTION",
        `utf8Eci conflicts with eci ${explicit}`,
      );
    }
    return UTF8_ECI;
  }
  return explicit ?? null;
}

function isSegmentList(value: QrCodeValue): value is readonly QrCodeSegment[] {
  return Array.isArray(value);
}

function dataModeOf(segments: readonly QrCodeSegment[]): QrCodeMatrix["mode"] {
  let mode: QrCodeMode | null = null;
  for (const segment of segments) {
    if (segment.mode === "eci") continue;
    if (mode !== null && mode !== segment.mode) return "mixed";
    mode = segment.mode;
  }
  return mode;
}

/**
 * Encode text, bytes, or explicit segments as an immutable QR Code symbol.
 *
 * Text is split into optimally mixed numeric, alphanumeric, byte (UTF-8), and
 * — with a `kanji` encoder — Kanji segments, re-optimized for each range of
 * character-count widths (versions 1–9, 10–26, 27–40). `segmentation:
 * "single"` or a forced `mode` keeps one segment. An `eci` / `utf8Eci` header
 * precedes the data when requested. The smallest fitting version is chosen
 * unless `version` fixes one, and the mask with the lowest penalty is applied
 * unless `mask` fixes one. Pure and deterministic, so it is safe during SSR.
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_DATA_TOO_LONG` when the value does
 * not fit the allowed versions, `VIZE_UI_QR_INVALID_MODE` when a forced mode
 * cannot represent it, or `VIZE_UI_QR_INVALID_OPTION` for out-of-range options.
 */
export function encodeQrCode(value: QrCodeValue, options: QrCodeEncodeOptions = {}): QrCodeMatrix {
  const requestedEcc = options.errorCorrection ?? "M";
  if (!QR_CODE_ERROR_CORRECTION_LEVELS.includes(requestedEcc)) {
    throw new QrCodeEncodeError(
      "VIZE_UI_QR_INVALID_OPTION",
      "errorCorrection must be L, M, Q, or H",
    );
  }
  const fixedVersion = options.version ?? "auto";
  let minVersion: QrCodeVersion;
  let maxVersion: QrCodeVersion;
  if (fixedVersion === "auto") {
    minVersion = readVersionOption("minVersion", options.minVersion, 1);
    maxVersion = readVersionOption("maxVersion", options.maxVersion, 40);
  } else {
    minVersion = readVersionOption("version", fixedVersion, 1);
    maxVersion = minVersion;
  }
  if (minVersion > maxVersion) {
    throw new QrCodeEncodeError(
      "VIZE_UI_QR_INVALID_OPTION",
      "minVersion must not exceed maxVersion",
    );
  }
  const maskOption = options.mask ?? "auto";
  if (maskOption !== "auto" && !isMask(maskOption)) {
    throw new QrCodeEncodeError("VIZE_UI_QR_INVALID_OPTION", "mask must be an integer in 0..7");
  }
  const segmentation = options.segmentation ?? "optimal";
  if (segmentation !== "optimal" && segmentation !== "single") {
    throw new QrCodeEncodeError(
      "VIZE_UI_QR_INVALID_OPTION",
      "segmentation must be optimal or single",
    );
  }
  const eci = readEciOption(options);
  const header = eci === null ? [] : [createQrCodeEciSegment(eci)];

  // Optimal segmentation depends on count-indicator widths, which change at
  // versions 10 and 27, so segments are recomputed when entering a new range.
  const segmentsFor = (candidate: QrCodeVersion): readonly QrCodeSegment[] => {
    if (isSegmentList(value)) return [...header, ...value];
    const data = createQrCodeSegments(value, {
      kanji: options.kanji,
      mode: options.mode ?? "auto",
      segmentation,
      version: candidate,
    });
    return [...header, ...data];
  };
  const rangeStarts = new Set([minVersion, 10, 27]);
  let segments: readonly QrCodeSegment[] = [];
  let version: QrCodeVersion | null = null;
  let usedBits = 0;
  for (let candidate: number = minVersion; candidate <= maxVersion; candidate++) {
    if (!isVersion(candidate)) break;
    if (rangeStarts.has(candidate)) segments = segmentsFor(candidate);
    const bits = qrCodeSegmentBitLength(segments, candidate);
    if (bits <= dataCodewordCount(candidate, requestedEcc) * 8) {
      version = candidate;
      usedBits = bits;
      break;
    }
  }
  if (version === null) {
    throw new QrCodeEncodeError(
      "VIZE_UI_QR_DATA_TOO_LONG",
      `value does not fit version ${maxVersion} at error correction ${requestedEcc}`,
    );
  }

  let ecc = requestedEcc;
  if (options.boostErrorCorrection === true) {
    for (const level of QR_CODE_ERROR_CORRECTION_LEVELS) {
      if (usedBits <= dataCodewordCount(version, level) * 8) ecc = level;
    }
  }

  const data = createDataCodewords(segments, version, ecc);
  const codewords = addErrorCorrectionAndInterleave(data, version, ecc);
  const canvas = new SymbolCanvas(version);
  canvas.drawFunctionPatterns();
  canvas.drawCodewords(codewords);

  let mask: QrCodeMask;
  if (maskOption === "auto") {
    mask = 0;
    let lowest = Number.POSITIVE_INFINITY;
    for (const candidate of MASKS) {
      canvas.applyMask(candidate);
      canvas.drawFormat(formatInformationBits(ecc, candidate));
      const penalty = qrCodePenaltyScore(canvas.size, canvas.modules);
      if (penalty < lowest) {
        mask = candidate;
        lowest = penalty;
      }
      canvas.applyMask(candidate);
    }
  } else {
    mask = maskOption;
  }
  canvas.applyMask(mask);
  canvas.drawFormat(formatInformationBits(ecc, mask));

  return Object.freeze({
    version,
    size: canvas.size,
    errorCorrection: ecc,
    mask,
    mode: dataModeOf(segments),
    segments: Object.freeze(
      segments.map((segment) =>
        Object.freeze({
          mode: segment.mode,
          count: segment.mode === "eci" ? eciDesignatorOf(segment) : segment.count,
        }),
      ),
    ),
    eci,
    modules: Object.freeze([...canvas.modules]),
  });
}

/** Decode the designator from an ECI segment's 8/16/24 payload bits. */
function eciDesignatorOf(segment: QrCodeSegment): number {
  const prefix = segment.bits[0] === 0 ? 1 : segment.bits[1] === 0 ? 2 : 3;
  let value = 0;
  for (const bit of segment.bits.slice(prefix)) value = value * 2 + bit;
  return value;
}

/** Whether the module at `(x, y)` is dark. Coordinates outside the symbol are light. */
export function isQrCodeModuleDark(matrix: QrCodeMatrix, x: number, y: number): boolean {
  if (x < 0 || y < 0 || x >= matrix.size || y >= matrix.size) return false;
  return matrix.modules[y * matrix.size + x] === true;
}

/**
 * Maximum characters a mode holds at a version and level. Useful for input
 * limits and for choosing a stronger level when a logo covers the symbol.
 */
export function qrCodeCapacity(
  version: QrCodeVersion,
  errorCorrection: QrCodeErrorCorrection,
  mode: QrCodeMode,
): number {
  const available =
    dataCodewordCount(version, errorCorrection) * 8 - 4 - characterCountBits(mode, version);
  const maxCount = 2 ** characterCountBits(mode, version) - 1;
  let count: number;
  switch (mode) {
    case "numeric": {
      const groups = Math.floor(available / 10);
      const rest = available - groups * 10;
      count = groups * 3 + (rest >= 7 ? 2 : rest >= 4 ? 1 : 0);
      break;
    }
    case "alphanumeric": {
      const pairs = Math.floor(available / 11);
      count = pairs * 2 + (available - pairs * 11 >= 6 ? 1 : 0);
      break;
    }
    case "byte":
      count = Math.floor(available / 8);
      break;
    case "kanji":
      count = Math.floor(available / 13);
      break;
  }
  return Math.min(count, maxCount);
}
