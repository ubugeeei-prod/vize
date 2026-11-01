import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Fixture, SourceFile } from "./model.ts";
import { array, bytes, hash, object, sha256, sourcePath, text } from "./value.ts";
export const reportHashes = {
  11252: "d4e14364467bbb5c3c346ae19db550095938e4fe0e82019bea59d136ecbea67d",
  11253: "0e889a4a47224eed93c5fe809f431a97b804c8a86b7c089540bb52d771a61284",
} as const;
const nullPins = [
  [
    "Box.vue.txt",
    "src/Box.vue",
    123,
    "ed94abad4d158f959ca0358fbc40484ea6e4877c033ba5885975298426a9efc7",
  ],
  [
    "package.json.txt",
    "package.json",
    299,
    "ae354c1bfea4681361209a04e013e1419bf5b9c21a27b29354184abd4b6cb716",
  ],
  [
    "tsconfig.json.txt",
    "tsconfig.json",
    191,
    "2a0b4a22837c11e90687de22c38b0c5283fffd39d6aa904f9741342883f7a47f",
  ],
] as const;
export const mixedInputSha = "41f272617bd60acc5c737f4d72327edcc0e85e53ca161ef076ca8509072c1ed6";
export const nullReceiptSha = "42a655bad3104e3665d4d75c540f008812e9dfd42cf11fcd330e1282b671f2a4";
function image(path: string, raw: Buffer): SourceFile {
  sourcePath(path);
  return { path, base64: raw.toString("base64"), sha256: sha256(raw) };
}
export function loadNull(root: string, receiptPath: string): Fixture {
  const rawReceipt = readFileSync(receiptPath),
    receipt = object(JSON.parse(rawReceipt.toString("utf8")));
  assert.equal(sha256(rawReceipt), nullReceiptSha);
  assert.equal(receipt.issue, 11252);
  assert.equal(receipt.reportSha256, reportHashes[11252]);
  assert.deepEqual(
    array(receipt.fixtures),
    nullPins.map(([file, , count, sha]) => ({ file, bytes: count, sha256: sha })),
  );
  return {
    issue: 11252,
    receiptSha256: sha256(rawReceipt),
    files: nullPins.map(([file, target, count, sha]) => {
      const raw = readFileSync(join(root, file));
      assert.equal(raw.length, count);
      assert.equal(sha256(raw), sha);
      return image(target, raw);
    }),
  };
}
export function loadOriginalFixture(
  role: "runtime-prop-null-11252" | "imports-vue-ts-11253",
): Fixture {
  const inputs = object(
    JSON.parse(readFileSync(new URL("./original-inputs.json", import.meta.url), "utf8")),
  );
  assert.equal(inputs.schema, "vize.humanReports11252_11253.originalInputs.v1");
  assert.equal(inputs.nullSourceHead, "5c7ab4d77c8f25c0a2d05522b391454dd9389039");
  assert.equal(inputs.mixedSourceHead, "4949b9f9f735f84de37fc9865fa65986fee37e1f");
  if (role === "imports-vue-ts-11253") return mixedBytes(bytes(text(inputs.mixedInputBase64)));
  const originals = array(inputs.nullFiles).map(object);
  assert.equal(originals.length, nullPins.length);
  return {
    issue: 11252,
    receiptSha256: nullReceiptSha,
    files: nullPins.map(([file, target, count, sha]) => {
      const rows = originals.filter((v) => v.file === file);
      assert.equal(rows.length, 1);
      const raw = bytes(text(rows[0]?.rawBase64));
      assert.equal(raw.length, count);
      assert.equal(sha256(raw), sha);
      return image(target, raw);
    }),
  };
}
export function loadMixed(inputPath: string): Fixture {
  return mixedBytes(readFileSync(inputPath));
}
function mixedBytes(raw: Buffer): Fixture {
  assert.equal(sha256(raw), mixedInputSha);
  const input = object(JSON.parse(raw.toString("utf8")));
  assert.equal(input.schema, "vize.mixed-vue-package-imports");
  assert.equal(input.id, "typechecker/regression/mixed-vue-package-imports-11253");
  const files = array(input.files).map((v) => {
    const row = object(v);
    return image(text(row.path), Buffer.from(text(row.source)));
  });
  assert.deepEqual(
    files.map((f) => f.path),
    ["package.json", "tsconfig.json", "src/App.vue", "src/main.ts", "src/plain.ts"],
  );
  return { issue: 11253, receiptSha256: mixedInputSha, files };
}
export function validateFiles(files: SourceFile[]): void {
  assert(files.length > 0);
  assert.equal(new Set(files.map((f) => f.path)).size, files.length);
  for (const file of files) {
    sourcePath(file.path);
    assert.equal(sha256(bytes(file.base64)), hash(file.sha256));
  }
}
export function assertOriginal(fixture: Fixture, expected: Fixture): void {
  validateFiles(fixture.files);
  assert.deepEqual(fixture, expected, "whole immutable original fixture differs");
}
