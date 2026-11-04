import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export const sourceAriaSharedImport = 'import type {} from "vize/virtual-ts-checker";';

export function sourceAriaDeclarationFiles(manifest: {
  files?: unknown;
  exports?: Record<string, unknown>;
}): string[] {
  assert.ok(Array.isArray(manifest.files));
  assert.ok(manifest.files.every((file) => typeof file === "string"));
  const files = manifest.files.filter((file: string) => /\.d\.(?:ts|mts|cts)$/u.test(file));
  assert.equal(new Set(files).size, files.length);
  assert.ok(
    files.includes("virtual-ts-checker.d.ts"),
    "source SDK must include its checker module",
  );
  assert.deepEqual(manifest.exports?.["./virtual-ts-checker"], {
    types: "./virtual-ts-checker.d.ts",
  });
  return files;
}

export function copyAriaDeclarations(source: string, destination: string, files: string[]) {
  const image = files.map((file) => ({ file, bytes: fs.readFileSync(path.join(source, file)) }));
  for (const { file, bytes } of image) {
    const target = path.join(destination, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes);
  }
}
