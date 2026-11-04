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
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { hasFileMetadataChanged } from "./state.ts";
import { PRECOMPILE_CACHE_ENV } from "./precompile-cache.ts";
import { compileAll } from "./precompile-run.ts";
import {
  BASE_SOURCE,
  coldRun,
  makeProject,
  makeState,
  manifestFiles,
  persistedFiles,
  recompiledCount,
  restoredCount,
} from "../test/precompile-cache-project.ts";

/**
 * Two cold runs of `root` in a child process. The kill switch is a process-wide
 * env var and warnings reach the process console, while the sibling suites run
 * concurrently in this one (#11566), so neither may be changed here.
 */
function isolatedColdRuns(root: string, env: NodeJS.ProcessEnv = {}) {
  const helpers = JSON.stringify(
    new URL("../test/precompile-cache-project.ts", import.meta.url).href,
  );
  const script = `
const { coldRun, manifestFiles } = await import(${helpers});
const runs = [];
for (let i = 0; i < 2; i++) {
  const warnings = [];
  console.warn = (...args) => warnings.push(args.map(String).join(" "));
  const { log } = await coldRun(${JSON.stringify(root)});
  runs.push({ log, warnings, manifests: manifestFiles(${JSON.stringify(root)}).length });
}
process.stdout.write(JSON.stringify(runs));`;
  const childEnv = { ...process.env };
  delete childEnv[PRECOMPILE_CACHE_ENV];
  const child = spawnSync(
    process.execPath,
    ["--no-warnings", "--input-type=module", "-e", script],
    {
      env: { ...childEnv, ...env },
      encoding: "utf8",
    },
  );
  assert.equal(child.status, 0, child.stderr);
  const runs: { log: string; warnings: string[]; manifests: number }[] = JSON.parse(child.stdout);
  return runs;
}

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
// A cold restore must not hide warnings from an unchanged source.
// ---------------------------------------------------------------------------
{
  const { root } = makeProject(`<script setup>
import { reactive } from "vue";
const form = reactive({ name: "" });
</script>
<template><input v-model="form" /></template>`);
  const [first, second] = isolatedColdRuns(root);
  assert.equal(recompiledCount(first.log), 1);
  assert.ok(first.warnings.some((message) => message.includes("v-model")));
  assert.equal(restoredCount(second.log), 0, "warned sources cannot restore silently");
  assert.equal(recompiledCount(second.log), 1);
  assert.ok(second.warnings.some((message) => message.includes("v-model")));
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
// Project captures are process-local: cold runs recompile and register watches.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(
    `<script setup lang="ts">import type { Props } from './Props';defineProps<Props>();</script>`,
  );
  const config = path.join(root, "tsconfig.json");
  const types = path.join(root, "src", "Props.ts");
  fs.writeFileSync(config, "{}");
  for (const constructor of ["String", "Number"]) {
    fs.writeFileSync(types, `export interface Props { value?: ${constructor.toLowerCase()} }`);
    const lines: string[] = [];
    const state = makeState(root, {}, (line) => lines.push(line));
    const watches: string[] = [];
    await compileAll(state, [], undefined, (input) => watches.push(input));
    assert.equal(restoredCount(lines.join("\n")), 0);
    assert.equal(recompiledCount(lines.join("\n")), 1);
    const compiled = state.cache.get(file)!;
    assert.ok(compiled.typeProject, "the fresh compile owns a live project capture");
    assert.ok(watches.includes(config), "each cold run registers configuration watches");
    assert.deepEqual(
      compiled.dependencies?.toSorted(),
      [config, types, path.join(root, "src", "tsconfig.json")].toSorted(),
    );
    if (manifestFiles(root).length > 0) {
      assert.equal(persistedFiles(root).includes(file), false);
    }
  }
}

// ---------------------------------------------------------------------------
// Env kill switch: no reads, no writes.
// ---------------------------------------------------------------------------
{
  const [first, second] = isolatedColdRuns(makeProject(BASE_SOURCE).root, {
    [PRECOMPILE_CACHE_ENV]: "0",
  });
  assert.equal(first.manifests, 0, "a disabled cache must not write a manifest");
  assert.equal(restoredCount(second.log), 0, "a disabled cache must never restore");

  // Control: the same isolated runs, cache on, write and restore.
  const [cold, warm] = isolatedColdRuns(makeProject(BASE_SOURCE).root);
  assert.equal(cold.manifests, 1);
  assert.equal(restoredCount(warm.log), 1);
}

console.log("✅ vite-plugin-vize precompile cache staleness tests passed!");
