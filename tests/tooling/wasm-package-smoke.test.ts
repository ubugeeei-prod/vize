import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { smokeWasmPackage } from "../../tools/npm/smoke-wasm-package.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function copyRepoFile(targetDir: string, relativePath: string): void {
  fs.copyFileSync(path.join(root, relativePath), path.join(targetDir, path.basename(relativePath)));
}

test("wasm package smoke validates manifest, files, exports, and init guard", async () => {
  const packageDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-smoke-"));

  copyRepoFile(packageDir, "npm/vize-wasm/package.json");
  copyRepoFile(packageDir, "npm/vize-wasm/index.js");
  copyRepoFile(packageDir, "npm/vize-wasm/index.d.ts");
  fs.writeFileSync(path.join(packageDir, "vize_vitrine.d.ts"), "export {};\n");
  fs.writeFileSync(path.join(packageDir, "vize_vitrine_bg.wasm"), "");
  fs.writeFileSync(
    path.join(packageDir, "vize_vitrine.js"),
    `export default async function init() {}
export class Compiler {
  compile() {}
  compileVapor() {}
  parse() {}
  parseSfc() {}
  compileSfc() {}
  compileCss() {}
  free() {}
}
export function compile() {}
export function compileVapor() {}
export function parseTemplate() {}
export function parseSfc() {}
export function compileSfc() {}
export function compileCss() {}
`,
  );

  await smokeWasmPackage(packageDir);
});

test("wasm package smoke fails when wrapper entrypoint is missing", async () => {
  const packageDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-smoke-missing-"));

  copyRepoFile(packageDir, "npm/vize-wasm/package.json");
  fs.writeFileSync(path.join(packageDir, "index.d.ts"), "export {};\n");
  fs.writeFileSync(
    path.join(packageDir, "vize_vitrine.js"),
    "export default async function init() {}\n",
  );
  fs.writeFileSync(path.join(packageDir, "vize_vitrine.d.ts"), "export {};\n");
  fs.writeFileSync(path.join(packageDir, "vize_vitrine_bg.wasm"), "");

  await assert.rejects(() => smokeWasmPackage(packageDir), /index\.js is missing/);
});
