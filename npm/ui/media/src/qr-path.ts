/** UTF-8 text or raw bytes accepted by the QR encoder. */
export type QRCodeValue = string | readonly number[];

/** QR error-correction level, ordered from lowest to highest recovery. */
export type QRErrorCorrection = "L" | "M" | "Q" | "H";

/**
 * Converts a square QR boolean matrix to a compact SVG path.
 *
 * Adjacent dark modules on the same row are emitted as one path segment to
 * reduce both output size and renderer work.
 *
 * @throws {RangeError} When the matrix is empty or is not square.
 */
export function createQRPath(matrix: readonly (readonly boolean[])[]): string {
  const size = matrix.length;
  if (size === 0) {
    throw new RangeError("[VIZE_UI_MEDIA_INVALID_QR_MATRIX] QR matrix must not be empty");
  }

  const commands: string[] = [];
  for (let y = 0; y < size; y += 1) {
    const row = matrix[y];
    if (row === undefined || row.length !== size) {
      throw new RangeError("[VIZE_UI_MEDIA_INVALID_QR_MATRIX] QR matrix must be square");
    }

    let runStart = -1;
    for (let x = 0; x <= size; x += 1) {
      if (row[x] === true) {
        if (runStart === -1) runStart = x;
        continue;
      }

      if (runStart === -1) continue;
      commands.push(`M${runStart} ${y}h${x - runStart}v1H${runStart}z`);
      runStart = -1;
    }
  }

  return commands.join("");
}
