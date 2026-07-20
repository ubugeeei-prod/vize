import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { encode } from "uqr";
import { createQRPath } from "./qr-path.ts";

void test("combines adjacent dark modules into compact SVG path segments", () => {
  assert.equal(
    createQRPath([
      [true, true, false, true],
      [false, true, true, false],
      [false, false, false, false],
      [true, true, true, true],
    ]),
    "M0 0h2v1H0zM3 0h1v1H3zM1 1h2v1H1zM0 3h4v1H0z",
  );
});

void test("creates a path for an encoded symbol without adding an implicit border", () => {
  const symbol = encode("https://vize.dev", { border: 0, ecc: "H" });
  const path = createQRPath(symbol.data);

  assert.equal(symbol.data.length, symbol.size);
  assert.match(path, /^M/);
  assert.ok(path.length > symbol.size);
});

void test("rejects empty and non-square matrices", () => {
  assert.throws(() => createQRPath([]), /\[VIZE_UI_MEDIA_INVALID_QR_MATRIX\]/);
  assert.throws(() => createQRPath([[true], [false, true]]), /\[VIZE_UI_MEDIA_INVALID_QR_MATRIX\]/);
});

void test("keeps the QR component on the accessible SFC contract", async () => {
  const source = await readFile(new URL("./QRCode.vue", import.meta.url), "utf8");

  assert.match(source, /<script setup lang="ts">/);
  assert.match(source, /<title :id="titleId">/);
  assert.match(source, /role="img"/);
  assert.match(source, /defineExpose\(\{ element \}\)/);
  assert.doesNotMatch(source, /\bh\s*\(|defineOptions|withDefaults|interface (?:Props|Emits)/);
});
