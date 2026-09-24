import type { QrCodeErrorCorrection, QrCodeMode, QrCodeVersion } from "./qr-code-types.ts";

/** Error correction levels from weakest to strongest. */
export const QR_CODE_ERROR_CORRECTION_LEVELS: readonly QrCodeErrorCorrection[] = Object.freeze([
  "L",
  "M",
  "Q",
  "H",
]);

const ECC_INDEX: Readonly<Record<QrCodeErrorCorrection, number>> = Object.freeze({
  L: 0,
  M: 1,
  Q: 2,
  H: 3,
});

/** Two-bit format field value for each level (ISO/IEC 18004 table 12). */
export const QR_CODE_FORMAT_BITS: Readonly<Record<QrCodeErrorCorrection, number>> = Object.freeze({
  L: 1,
  M: 0,
  Q: 3,
  H: 2,
});

// ISO/IEC 18004 table 9. Index 0 is padding so a version indexes directly.
// prettier-ignore
const ECC_CODEWORDS_PER_BLOCK: readonly (readonly number[])[] = [
  [-1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
  [-1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
  [-1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
  [-1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
];

// prettier-ignore
const ERROR_CORRECTION_BLOCKS: readonly (readonly number[])[] = [
  [-1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
  [-1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49],
  [-1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68],
  [-1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81],
];

/** Mode indicator nibble for each supported mode. */
export const QR_CODE_MODE_INDICATOR: Readonly<Record<QrCodeMode, number>> = Object.freeze({
  numeric: 0x1,
  alphanumeric: 0x2,
  byte: 0x4,
  kanji: 0x8,
});

/** Mode indicator nibble for an ECI header segment. */
export const QR_CODE_ECI_MODE_INDICATOR = 0x7;

const CHARACTER_COUNT_BITS: Readonly<Record<QrCodeMode, readonly [number, number, number]>> =
  Object.freeze({
    numeric: [10, 12, 14],
    alphanumeric: [9, 11, 13],
    byte: [8, 16, 16],
    kanji: [8, 10, 12],
  });

function tableValue(
  table: readonly (readonly number[])[],
  ecc: QrCodeErrorCorrection,
  version: number,
): number {
  return table[ECC_INDEX[ecc]]?.[version] ?? 0;
}

/** Error correction codewords in each block. */
export function eccCodewordsPerBlock(version: QrCodeVersion, ecc: QrCodeErrorCorrection): number {
  return tableValue(ECC_CODEWORDS_PER_BLOCK, ecc, version);
}

/** Number of error correction blocks the codewords are split into. */
export function errorCorrectionBlockCount(
  version: QrCodeVersion,
  ecc: QrCodeErrorCorrection,
): number {
  return tableValue(ERROR_CORRECTION_BLOCKS, ecc, version);
}

/** Modules per side for a version. */
export function qrCodeSizeOf(version: QrCodeVersion): number {
  return version * 4 + 17;
}

/**
 * Data and error correction modules available after function patterns,
 * including remainder bits.
 */
export function rawDataModuleCount(version: QrCodeVersion): number {
  let result = (16 * version + 128) * version + 64;
  if (version >= 2) {
    const alignmentCount = Math.floor(version / 7) + 2;
    result -= (25 * alignmentCount - 10) * alignmentCount - 55;
    if (version >= 7) result -= 36;
  }
  return result;
}

/** 8-bit data codewords available for a version and level. */
export function dataCodewordCount(version: QrCodeVersion, ecc: QrCodeErrorCorrection): number {
  return (
    Math.floor(rawDataModuleCount(version) / 8) -
    eccCodewordsPerBlock(version, ecc) * errorCorrectionBlockCount(version, ecc)
  );
}

/** Character count indicator width for a mode at a version. */
export function characterCountBits(mode: QrCodeMode, version: QrCodeVersion): number {
  const widths = CHARACTER_COUNT_BITS[mode];
  return widths[version <= 9 ? 0 : version <= 26 ? 1 : 2];
}

/** Center coordinates of alignment patterns on each axis, in ascending order. */
export function alignmentPatternPositions(version: QrCodeVersion): readonly number[] {
  if (version === 1) return [];
  const count = Math.floor(version / 7) + 2;
  const step = Math.floor((version * 8 + count * 3 + 5) / (count * 4 - 4)) * 2;
  const positions = [6];
  for (let position = qrCodeSizeOf(version) - 7; positions.length < count; position -= step) {
    positions.splice(1, 0, position);
  }
  return positions;
}
