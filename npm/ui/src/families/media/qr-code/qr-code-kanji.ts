/**
 * Opt-in JIS X 0208 Shift_JIS table for QR Kanji mode.
 *
 * Kept out of the `qr-code` entry so the default encoder stays small: import
 * `qrCodeKanjiEncoder` and pass it as the `kanji` option (or `QrCode` prop)
 * only where Japanese text should use 13-bit Kanji segments.
 */
import { QR_CODE_KANJI_TABLE, QR_CODE_KANJI_TABLE_LENGTH } from "./qr-code-kanji-table.ts";
import type { QrCodeKanjiEncoder } from "./qr-code-types.ts";

const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * JIS (JIS X 0208 / JIS X 0221) mappings that differ from the Microsoft code
 * page 932 mappings the table is generated from. Both spellings encode.
 */
const JIS_ALIASES: readonly (readonly [codePoint: number, shiftJis: number])[] = [
  [0x301c, 0x8160], // WAVE DASH, CP932 uses FULLWIDTH TILDE
  [0x2016, 0x8161], // DOUBLE VERTICAL LINE, CP932 uses PARALLEL TO
  [0x2212, 0x817c], // MINUS SIGN, CP932 uses FULLWIDTH HYPHEN-MINUS
  [0x00a2, 0x8191], // CENT SIGN, CP932 uses FULLWIDTH CENT SIGN
  [0x00a3, 0x8192], // POUND SIGN, CP932 uses FULLWIDTH POUND SIGN
  [0x00ac, 0x81ca], // NOT SIGN, CP932 uses FULLWIDTH NOT SIGN
];

let reverse: ReadonlyMap<number, number> | null = null;

function decodeBase64(text: string): Uint8Array {
  const bytes = new Uint8Array(Math.floor((text.length * 3) / 4));
  let length = 0;
  let buffer = 0;
  let bits = 0;
  for (const character of text) {
    const value = BASE64.indexOf(character);
    if (value < 0) continue;
    buffer = (buffer << 6) | value;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes[length++] = (buffer >>> bits) & 0xff;
    }
  }
  return bytes.subarray(0, length);
}

/** Shift_JIS code for a 13-bit QR Kanji value. */
function shiftJisOf(value: number): number {
  const offset = (Math.floor(value / 0xc0) << 8) | (value % 0xc0);
  return offset + (offset >= 0x1f00 ? 0xc140 : 0x8140);
}

function buildReverseTable(): ReadonlyMap<number, number> {
  const bytes = decodeBase64(QR_CODE_KANJI_TABLE);
  const table = new Map<number, number>();
  let position = 0;
  let previous = 0;
  for (let value = 0; value < QR_CODE_KANJI_TABLE_LENGTH; value++) {
    let zigzag = 0;
    let shift = 0;
    let byte: number;
    do {
      byte = bytes[position++] ?? 0;
      zigzag += (byte & 0x7f) * 2 ** shift;
      shift += 7;
    } while (byte & 0x80);
    previous += zigzag % 2 === 0 ? zigzag / 2 : -(zigzag + 1) / 2;
    // The first (lowest) Shift_JIS code wins for duplicated characters.
    if (previous !== 0 && !table.has(previous)) table.set(previous, shiftJisOf(value));
  }
  for (const [codePoint, shiftJis] of JIS_ALIASES) {
    if (!table.has(codePoint)) table.set(codePoint, shiftJis);
  }
  return table;
}

/**
 * JIS X 0208 Shift_JIS encoder for QR Kanji mode. The packed table is decoded
 * once, on first use.
 */
export const qrCodeKanjiEncoder: QrCodeKanjiEncoder = Object.freeze({
  toShiftJis(codePoint: number): number | undefined {
    reverse ??= buildReverseTable();
    return reverse.get(codePoint);
  },
});
