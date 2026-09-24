import { QrCodeEncodeError } from "./qr-code-encoder.ts";
import type { QrCodeMatrix, QrCodeSvgPathOptions } from "./qr-code-types.ts";

/**
 * Validate a quiet zone width. The standard requires 4 modules; smaller
 * values are allowed for layouts that supply their own light margin.
 *
 * @throws {QrCodeEncodeError} `VIZE_UI_QR_INVALID_OPTION` for negative or
 * non-integer widths.
 */
export function normalizeQrCodeQuietZone(quietZone: number): number {
  if (!Number.isInteger(quietZone) || quietZone < 0 || quietZone > 64) {
    throw new QrCodeEncodeError(
      "VIZE_UI_QR_INVALID_OPTION",
      "quietZone must be an integer in 0..64",
    );
  }
  return quietZone;
}

/**
 * Build one deterministic SVG path `d` string covering every dark module.
 *
 * Horizontal runs of dark modules are merged into a single rectangle each
 * (`M x y h n v1 h -n z`), so the output stays compact and renders without
 * hairline seams under `shape-rendering="crispEdges"`. Coordinates are in
 * module units, offset by the quiet zone, matching a
 * `viewBox="0 0 size+2q size+2q"`.
 */
export function qrCodeToSvgPath(matrix: QrCodeMatrix, options: QrCodeSvgPathOptions = {}): string {
  const offset = normalizeQrCodeQuietZone(options.quietZone ?? 0);
  const { size, modules } = matrix;
  const commands: string[] = [];
  for (let y = 0; y < size; y++) {
    let x = 0;
    while (x < size) {
      if (modules[y * size + x] !== true) {
        x++;
        continue;
      }
      const start = x;
      while (x < size && modules[y * size + x] === true) x++;
      const length = x - start;
      commands.push(`M${start + offset} ${y + offset}h${length}v1h-${length}z`);
    }
  }
  return commands.join("");
}
