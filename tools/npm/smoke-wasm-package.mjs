#!/usr/bin/env node
/**
 * Smoke-test the publishable @vizejs/wasm package directory.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const REQUIRED_FILES = [
  "index.js",
  "index.d.ts",
  "vize_vitrine.js",
  "vize_vitrine.d.ts",
  "vize_vitrine_bg.wasm",
];

const REQUIRED_EXPORTS = [
  "Compiler",
  "compile",
  "compileCss",
  "compileSfc",
  "compileVapor",
  "default",
  "init",
  "isInitialized",
  "parseSfc",
  "parseTemplate",
];

function requireArg(argv) {
  const packageDir = argv[0];
  if (!packageDir) {
    throw new Error("Usage: smoke-wasm-package.mjs <npm/vize-wasm>");
  }
  return path.resolve(packageDir);
}

function readPackageJson(packageDir) {
  return JSON.parse(fs.readFileSync(path.join(packageDir, "package.json"), "utf8"));
}

function assertManifest(packageJson) {
  assert.equal(packageJson.name, "@vizejs/wasm");
  assert.equal(packageJson.type, "module");
  assert.equal(packageJson.main, "./index.js");
  assert.equal(packageJson.types, "./index.d.ts");
  assert.equal(packageJson.exports?.["."]?.import, "./index.js");
  assert.equal(packageJson.exports?.["."]?.types, "./index.d.ts");
  assert.equal(packageJson.exports?.["./vize_vitrine.js"]?.import, "./vize_vitrine.js");
  assert.equal(packageJson.exports?.["./vize_vitrine.js"]?.types, "./vize_vitrine.d.ts");
  assert.equal(packageJson.exports?.["./vize_vitrine_bg.wasm"], "./vize_vitrine_bg.wasm");

  for (const file of REQUIRED_FILES) {
    assert.ok(packageJson.files?.includes(file), `package files must include ${file}`);
  }
}

function assertFilesExist(packageDir) {
  for (const file of REQUIRED_FILES) {
    assert.ok(fs.existsSync(path.join(packageDir, file)), `${file} is missing`);
  }
}

async function assertEntryPoint(packageDir) {
  const entry = await import(
    `${pathToFileURL(path.join(packageDir, "index.js")).href}?smoke=${Date.now()}`
  );
  for (const exportName of REQUIRED_EXPORTS) {
    assert.ok(exportName in entry, `missing export ${exportName}`);
  }

  assert.equal(entry.default, entry.init);
  assert.equal(entry.isInitialized(), false);
  assert.throws(() => entry.compile("<div />"), /Call `await init\(\)` first/);
}

export async function smokeWasmPackage(packageDir) {
  const packageJson = readPackageJson(packageDir);
  assertManifest(packageJson);
  assertFilesExist(packageDir);
  await assertEntryPoint(packageDir);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await smokeWasmPackage(requireArg(process.argv.slice(2)));
    console.log("@vizejs/wasm package smoke passed");
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
