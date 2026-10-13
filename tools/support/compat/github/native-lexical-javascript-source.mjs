import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  authenticateJavaScriptSfcSource,
  javascriptSfcContextPaths,
} from "./javascript-sfc-source-context.mjs";

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const supplementalHelpers = [
  ...javascriptSfcContextPaths,
  "tools/support/compat/github/native-lexical-javascript-source.mjs",
];
export const nativeLexicalSourcePaths = (manifest, helpers) => [
  ...new Set([
    ...helpers,
    ...manifest.files.map(([file]) => file),
    ...manifest.corpusBodies.map(([file]) => file),
    "tests/performance/support/warm-type-backed-lexical-dollar-manifest.json",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/vize_canon/Cargo.toml",
    "crates/vize_canon/src/lib.rs",
    ...supplementalHelpers,
  ]),
];

/**
 * Raw copied metadata remains actual; only the unchanged lexical table reads
 * historical source. The historical view comes from the recorded JavaScript
 * SFC context, so the packet does not copy or pin the files that context names.
 */
export function validateNativeLexicalSources(output, metadata, manifest, helpers) {
  assert.deepEqual(
    metadata.copied.map((entry) => entry.file).sort((a, b) => a.localeCompare(b)),
    nativeLexicalSourcePaths(manifest, helpers).sort((a, b) => a.localeCompare(b)),
  );
  for (const name of [...helpers, ...supplementalHelpers]) {
    const actual = new URL(`../../../../${name}`, import.meta.url);
    assert.ok(fs.lstatSync(actual).isFile());
    const copied = path.join(output, "source", name);
    assert.ok(fs.lstatSync(copied).isFile());
    assert.equal(fs.lstatSync(copied).mode & 0o777, 0o644);
    assert.equal(fs.lstatSync(actual).mode & 0o777, 0o644);
    assert.equal(
      metadata.copied.find((entry) => entry.file === name).sha256,
      digest(fs.readFileSync(actual)),
    );
  }
  const context = authenticateJavaScriptSfcSource();
  for (const [file, , expected] of [...manifest.files, ...manifest.corpusBodies])
    assert.equal(digest(context.historicalAfter(file)), expected);
  return context.receipt;
}
