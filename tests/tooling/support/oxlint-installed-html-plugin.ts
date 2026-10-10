import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { findPackageJSON } from "node:module";
import { exactPath } from "./n8n-installed-authority.ts";

/** Locate the genuine ancestor import-only package without loading its code. */
export function installedBarePlugin(
  root: string,
  installRoot: string,
  packageDirectory: string,
  entry: string,
) {
  const filename = findPackageJSON("oxlint-plugin-vize", path.join(root, "package.json"));
  assert.ok(filename, "bare public plugin package must resolve from the actual input workspace");
  assert.equal(exactPath(filename, installRoot), path.join(packageDirectory, "package.json"));
  const manifest = JSON.parse(fs.readFileSync(filename, "utf8"));
  assert.equal(manifest.name, "oxlint-plugin-vize");
  assert.equal(manifest.type, "module");
  assert.deepEqual(Object.keys(manifest.exports["."]).sort(), ["import", "types"]);
  assert.equal(manifest.exports["."].import, "./dist/index.mjs");
  assert.equal(
    exactPath(path.join(packageDirectory, manifest.exports["."].import), packageDirectory),
    entry,
  );
  return entry;
}
