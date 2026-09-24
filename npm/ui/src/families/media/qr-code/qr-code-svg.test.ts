import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { encodeQrCode } from "./qr-code-encoder.ts";
import { normalizeQrCodeQuietZone, qrCodeToSvgPath } from "./qr-code-svg.ts";
import type { QrCodeMatrix } from "./qr-code-types.ts";

function matrixOf(rows: readonly string[]): QrCodeMatrix {
  return {
    version: 1,
    size: rows.length,
    errorCorrection: "L",
    mask: 0,
    mode: null,
    modules: rows.flatMap((row) => row.split("").map((cell) => cell === "#")),
  };
}

/** Rasterize path rectangles back into a module grid to prove exact coverage. */
function rasterize(path: string, dimension: number): readonly boolean[] {
  const grid = Array.from<boolean>({ length: dimension * dimension }).fill(false);
  for (const match of path.matchAll(/M(\d+) (\d+)h(\d+)v1h-(\d+)z/g)) {
    const [, x, y, width, back] = match.map(Number);
    assert.equal(width, back);
    for (let offset = 0; offset < (width ?? 0); offset++) {
      const index = (y ?? 0) * dimension + (x ?? 0) + offset;
      assert.equal(grid[index], false, "runs must not overlap");
      grid[index] = true;
    }
  }
  return grid;
}

test("merges horizontal dark runs into one rectangle per run", () => {
  const matrix = matrixOf(["##.#", "....", "####", ".#.."]);
  assert.equal(qrCodeToSvgPath(matrix), "M0 0h2v1h-2zM3 0h1v1h-1zM0 2h4v1h-4zM1 3h1v1h-1z");
  assert.equal(qrCodeToSvgPath(matrixOf(["..", ".."])), "");
});

test("offsets every coordinate by the quiet zone", () => {
  const matrix = matrixOf(["#."]);
  assert.equal(qrCodeToSvgPath(matrix, { quietZone: 4 }), "M4 4h1v1h-1z");
});

test("covers exactly the dark modules of an encoded symbol", () => {
  const matrix = encodeQrCode("https://vizejs.dev", { errorCorrection: "Q" });
  const quietZone = 2;
  const dimension = matrix.size + quietZone * 2;
  const grid = rasterize(qrCodeToSvgPath(matrix, { quietZone }), dimension);
  for (let y = 0; y < dimension; y++) {
    for (let x = 0; x < dimension; x++) {
      const inner =
        x >= quietZone &&
        y >= quietZone &&
        x < quietZone + matrix.size &&
        y < quietZone + matrix.size;
      const expected =
        inner && matrix.modules[(y - quietZone) * matrix.size + x - quietZone] === true;
      assert.equal(grid[y * dimension + x], expected, `${x},${y}`);
    }
  }
  assert.equal(qrCodeToSvgPath(matrix, { quietZone }), qrCodeToSvgPath(matrix, { quietZone }));
});

test("rejects invalid quiet zones", () => {
  assert.equal(normalizeQrCodeQuietZone(0), 0);
  assert.throws(() => normalizeQrCodeQuietZone(-1), /VIZE_UI_QR_INVALID_OPTION/);
  assert.throws(() => normalizeQrCodeQuietZone(1.5), /VIZE_UI_QR_INVALID_OPTION/);
  assert.throws(
    () => qrCodeToSvgPath(matrixOf(["#"]), { quietZone: 65 }),
    /VIZE_UI_QR_INVALID_OPTION/,
  );
});
