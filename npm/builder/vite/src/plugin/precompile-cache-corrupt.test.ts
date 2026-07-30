/**
 * Damaged-container tests, end to end through `compileAll`.
 *
 * `./precompile-cache-store.test.ts` proves the container decoder refuses every
 * shape of damage. This file proves the refusal actually reaches the build:
 * whatever is wrong with the bytes on disk, the pass still compiles the file and
 * still produces output. A stale or mis-sliced hit would be wrong output, which
 * is strictly worse than being slow.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import {
  PRECOMPILE_CACHE_FORMAT,
  decodePrecompileManifest,
  encodePrecompileManifest,
} from "./precompile-cache.ts";
import {
  BASE_SOURCE,
  coldRun,
  makeProject,
  manifestKey,
  manifestPath,
  patchManifestHeader,
  recompiledCount,
  restoredCount,
} from "../test/precompile-cache-project.ts";

// ---------------------------------------------------------------------------
// Header gates: a container that does not claim to be ours is not read.
// ---------------------------------------------------------------------------
{
  const { root } = makeProject(BASE_SOURCE);
  await coldRun(root);

  for (const [label, patch] of [
    // A manifest written by a different plugin/compiler build: same file name (as
    // if the key algorithm itself changed), older format stamp.
    ["an older manifest format", { format: PRECOMPILE_CACHE_FORMAT - 1 }],
    // A manifest whose recorded key does not match the key it is filed under.
    ["a key mismatch", { key: "not-the-real-key" }],
    // New in format 2: a body compressed with something this Node cannot read...
    ["an unreadable codec", { codec: "brotli" }],
    // ...and body lengths that no longer account for the file exactly.
    ["a body length mismatch", { payload: 1 }],
    ["a missing body length", { index: undefined }],
  ] as const) {
    // Each miss recompiles and rewrites a sound container, so every patch here
    // is applied to a freshly written one.
    patchManifestHeader(root, patch);
    const run = await coldRun(root);
    assert.equal(restoredCount(run.log), 0, `${label} must be ignored`);
    assert.equal(recompiledCount(run.log), 1, `${label} must fall back to compiling`);
  }
}

// ---------------------------------------------------------------------------
// Corruption: truncated, non-JSON, and bit-rotted containers degrade to compile.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  await coldRun(root);
  const target = manifestPath(root);
  const good = fs.readFileSync(target);
  const header = good.subarray(0, good.indexOf(0x0a) + 1);
  const flipped = Buffer.from(good);
  flipped[good.length - 1] = good.at(-1)! ^ 0xff;

  for (const [label, contents] of [
    ["truncated", good.subarray(0, Math.floor(good.length / 2))],
    ["empty", Buffer.alloc(0)],
    ["not json", Buffer.from("<html>nope</html>")],
    ["json but not an object", Buffer.from("[]")],
    ["null", Buffer.from("null")],
    ["header only", header],
    // New in format 2: a sound header over bodies that are not compressed data,
    // and a single flipped payload byte, which fails the codec's own checksum.
    ["garbage bodies", Buffer.concat([header, Buffer.alloc(good.length - header.length, 0x41)])],
    ["one flipped payload byte", flipped],
  ] as const) {
    fs.writeFileSync(target, contents);
    const run = await coldRun(root);
    assert.equal(restoredCount(run.log), 0, `${label} manifest must not be restored`);
    assert.equal(recompiledCount(run.log), 1, `${label} manifest must fall back to compiling`);
    assert.ok(run.state.cache.has(file), `${label} manifest must still yield compiled output`);
  }
}

// ---------------------------------------------------------------------------
// One malformed entry is dropped on its own; its neighbours still restore.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  const other = path.join(path.dirname(file), "Other.vue");
  fs.writeFileSync(other, `<template><span>other</span></template>\n`);
  await coldRun(root);

  const target = manifestPath(root);
  const key = manifestKey(root);
  const entries = decodePrecompileManifest(fs.readFileSync(target), { key, root })!;
  assert.equal(entries.size, 2, "both files must be persisted before the damage");
  entries.get(file)!.module.scopeId = 42 as unknown as string;
  fs.writeFileSync(target, encodePrecompileManifest({ key, root, entries }));

  const run = await coldRun(root);
  assert.equal(restoredCount(run.log), 1, "the sound entry must still be restored");
  assert.equal(recompiledCount(run.log), 1, "only the malformed entry must be recompiled");
  assert.equal(typeof run.state.cache.get(file)!.scopeId, "string", "and it recompiles cleanly");
}

console.log("✅ vite-plugin-vize precompile cache corruption tests passed!");
