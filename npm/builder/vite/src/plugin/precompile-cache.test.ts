/**
 * Persistent pre-compile cache tests.
 *
 * The point of this suite is the *miss* path: every way a cache entry can go
 * stale must force a recompile, because a stale hit serves wrong output. The
 * hit path is covered too, but on its own it would prove nothing.
 *
 * Damage to the container on disk is the neighbouring concern, and lives in
 * `./precompile-cache-corrupt.test.ts`.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { hasFileMetadataChanged } from "./state.ts";
import { PRECOMPILE_CACHE_ENV } from "./precompile-cache.ts";
import {
  BASE_SOURCE,
  coldRun,
  makeProject,
  manifestFiles,
  persistedFiles,
  recompiledCount,
  restoredCount,
} from "../test/precompile-cache-project.ts";

// ---------------------------------------------------------------------------
// Hit path: a second cold process reuses the manifest.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);

  const first = await coldRun(root);
  assert.equal(recompiledCount(first.log), 1, "first cold run must compile");
  assert.equal(restoredCount(first.log), 0, "first cold run has nothing to restore");
  assert.equal(manifestFiles(root).length, 1, "first cold run must persist one manifest");

  const second = await coldRun(root);
  assert.equal(restoredCount(second.log), 1, "second cold run must restore from disk");
  assert.equal(recompiledCount(second.log), 0, "second cold run must not recompile");
  assert.deepEqual(
    second.state.cache.get(file),
    first.state.cache.get(file),
    "restored module must be byte-identical to the compiled one",
  );
  assert.ok(second.state.precompileMetadata.has(file), "restore must repopulate scan metadata");
}

// ---------------------------------------------------------------------------
// Stale case 1: content changes while the size stays identical.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  // Pin mtime to an exact millisecond so it can be restored bit-for-bit later.
  const pinned = new Date(1_700_000_000_000);
  fs.utimesSync(file, pinned, pinned);
  const before = fs.statSync(file);

  const first = await coldRun(root);
  const compiled = first.state.cache.get(file)!;

  // Same byte length, different bytes, and mtime forced back to the original.
  const original = fs.readFileSync(file, "utf-8");
  const edited = original.replace('class="a">one', 'class="a">two');
  assert.equal(edited.length, original.length, "edit must preserve the file size");
  assert.notEqual(edited, original);
  fs.writeFileSync(file, edited);
  fs.utimesSync(file, pinned, pinned);
  const after = fs.statSync(file);
  assert.equal(
    hasFileMetadataChanged(
      { mtimeMs: before.mtimeMs, size: before.size },
      { mtimeMs: after.mtimeMs, size: after.size },
    ),
    false,
    "the mtime+size heuristic must be blind to this edit -- that is the point",
  );

  const second = await coldRun(root);
  assert.equal(
    restoredCount(second.log),
    0,
    "a same-size same-mtime content change must not hit the cache",
  );
  assert.equal(recompiledCount(second.log), 1, "the edited file must be recompiled");
  const recompiled = second.state.cache.get(file)!;
  assert.notEqual(recompiled.code, compiled.code, "recompiled output must reflect the new source");
  assert.match(recompiled.code, /two/, "recompiled output must contain the new text");
}

// ---------------------------------------------------------------------------
// Stale case 2: a compiler option that changes output changes the key.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  const first = await coldRun(root);
  const vdom = first.state.cache.get(file)!;

  const switched = await coldRun(root, { vapor: true });
  assert.equal(
    restoredCount(switched.log),
    0,
    "changing a compile option must not reuse the previous option's output",
  );
  assert.equal(recompiledCount(switched.log), 1);
  assert.notEqual(
    switched.state.cache.get(file)!.code,
    vdom.code,
    "vapor output must differ from vdom output",
  );
  assert.equal(manifestFiles(root).length, 2, "each option set gets its own manifest");

  // ...and the original option set still hits its own manifest.
  const backToVdom = await coldRun(root);
  assert.equal(restoredCount(backToVdom.log), 1, "the original manifest must still be valid");
  assert.equal(backToVdom.state.cache.get(file)!.code, vdom.code);
}

// ---------------------------------------------------------------------------
// Deleted files are pruned rather than accumulating forever.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  const other = path.join(path.dirname(file), "Other.vue");
  fs.writeFileSync(other, `<template><span>other</span></template>\n`);
  await coldRun(root);
  assert.deepEqual(persistedFiles(root), [file, other].sort(), "both files must be persisted");

  fs.rmSync(other);
  fs.writeFileSync(file, `${BASE_SOURCE}<!-- touched -->\n`);
  await coldRun(root);
  assert.deepEqual(persistedFiles(root), [file], "the deleted file must be pruned");
}

// ---------------------------------------------------------------------------
// `src` imports are deliberately never persisted: their inputs are not hashed.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(
    `<template><div>src import</div></template>\n<script src="./setup.js"></script>\n`,
  );
  const dependency = path.join(path.dirname(file), "setup.js");
  fs.writeFileSync(dependency, `export default { name: "one" };\n`);

  const first = await coldRun(root);
  assert.ok(first.state.cache.get(file)?.dependencies?.length, "the SFC must record a dependency");
  const second = await coldRun(root);
  assert.equal(
    restoredCount(second.log),
    0,
    "an SFC with `src` imports must never be restored from disk",
  );
  assert.equal(recompiledCount(second.log), 1);

  // Proof of why: the dependency can change with the SFC untouched.
  fs.writeFileSync(dependency, `export default { name: "two" };\n`);
  const third = await coldRun(root);
  assert.equal(restoredCount(third.log), 0);
  assert.match(third.state.cache.get(file)!.code, /two/, "the new dependency must be compiled in");
}

// ---------------------------------------------------------------------------
// Env kill switch: no reads, no writes.
// ---------------------------------------------------------------------------
{
  const { root } = makeProject(BASE_SOURCE);
  const previous = process.env[PRECOMPILE_CACHE_ENV];
  process.env[PRECOMPILE_CACHE_ENV] = "0";
  try {
    await coldRun(root);
    assert.deepEqual(manifestFiles(root), [], "a disabled cache must not write a manifest");
    const second = await coldRun(root);
    assert.equal(restoredCount(second.log), 0, "a disabled cache must never restore");
  } finally {
    if (previous === undefined) {
      delete process.env[PRECOMPILE_CACHE_ENV];
    } else {
      process.env[PRECOMPILE_CACHE_ENV] = previous;
    }
  }
}

console.log("✅ vite-plugin-vize precompile cache staleness tests passed!");
