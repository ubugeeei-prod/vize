/**
 * Persistent pre-compile cache tests.
 *
 * The point of this suite is the *miss* path: every way a cache entry can go
 * stale must force a recompile, because a stale hit serves wrong output. The
 * hit path is covered too, but on its own it would prove nothing.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  hasFileMetadataChanged,
  type VizePluginState,
} from "./state.ts";
import { compileAll } from "./precompile-run.ts";
import {
  PRECOMPILE_CACHE_DIR,
  PRECOMPILE_CACHE_ENV,
  PRECOMPILE_CACHE_FORMAT,
  hashPrecompileSource,
} from "./precompile-cache.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(__dirname, "../../../..");
const testRoot = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "tests",
  "vite-plugin-vize",
  "precompile-cache",
);
fs.rmSync(testRoot, { recursive: true, force: true });
fs.mkdirSync(testRoot, { recursive: true });

let caseId = 0;
function makeProject(source: string): { root: string; file: string } {
  const root = path.join(testRoot, `case-${++caseId}`);
  const srcDir = path.join(root, "src");
  fs.mkdirSync(srcDir, { recursive: true });
  const file = path.join(srcDir, "App.vue");
  fs.writeFileSync(file, source);
  return { root, file };
}

function makeState(root: string, options: VizePluginState["mergedOptions"] = {}): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: ["src/**/*.vue"],
    precompileBatchSize: DEFAULT_PRECOMPILE_BATCH_SIZE,
    ignorePatterns: [],
    mergedOptions: options,
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
  };
}

/** Runs one cold `compileAll` (fresh state == fresh process) and reports the log. */
async function coldRun(
  root: string,
  options: VizePluginState["mergedOptions"] = {},
): Promise<{ state: VizePluginState; log: string }> {
  const state = makeState(root, options);
  const lines: string[] = [];
  state.logger = {
    log() {},
    info: (...args: unknown[]) => lines.push(args.join(" ")),
    warn() {},
    error() {},
  } as never;
  await compileAll(state);
  return { state, log: lines.join("\n") };
}

function restoredCount(log: string): number {
  return Number(/(\d+) restored from disk/.exec(log)?.[1] ?? "-1");
}

function recompiledCount(log: string): number {
  return Number(/(\d+) recompiled/.exec(log)?.[1] ?? "-1");
}

function manifestFiles(root: string): string[] {
  const dir = path.join(root, PRECOMPILE_CACHE_DIR);
  return fs.existsSync(dir) ? fs.readdirSync(dir).filter((name) => name.endsWith(".json")) : [];
}

const BASE_SOURCE = `<template><div class="a">one</div></template>
<script setup>const label = "one";</script>
<style scoped>.a { color: red; }</style>
`;

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
// Stale case 3: a format/version bump abandons the manifest wholesale.
// ---------------------------------------------------------------------------
{
  const { root } = makeProject(BASE_SOURCE);
  await coldRun(root);
  const [name] = manifestFiles(root);
  const manifestPath = path.join(root, PRECOMPILE_CACHE_DIR, name!);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8")) as Record<string, unknown>;

  // A manifest written by a different plugin/compiler build: same file name (as
  // if the key algorithm itself changed), older format stamp.
  fs.writeFileSync(
    manifestPath,
    JSON.stringify({ ...manifest, format: PRECOMPILE_CACHE_FORMAT - 1 }),
  );
  const stale = await coldRun(root);
  assert.equal(restoredCount(stale.log), 0, "an older manifest format must be ignored");
  assert.equal(recompiledCount(stale.log), 1);

  // A manifest whose recorded key does not match the key it is filed under.
  fs.writeFileSync(manifestPath, JSON.stringify({ ...manifest, key: "not-the-real-key" }));
  const mismatched = await coldRun(root);
  assert.equal(restoredCount(mismatched.log), 0, "a key mismatch must be ignored");
  assert.equal(recompiledCount(mismatched.log), 1);
}

// ---------------------------------------------------------------------------
// Corruption: truncated, non-JSON, and malformed entries all degrade to compile.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  await coldRun(root);
  const [name] = manifestFiles(root);
  const manifestPath = path.join(root, PRECOMPILE_CACHE_DIR, name!);
  const good = fs.readFileSync(manifestPath, "utf-8");

  for (const [label, contents] of [
    ["truncated", good.slice(0, Math.floor(good.length / 2))],
    ["empty", ""],
    ["not json", "<html>nope</html>"],
    ["json but not an object", "[]"],
    ["null", "null"],
  ] as const) {
    fs.writeFileSync(manifestPath, contents);
    const run = await coldRun(root);
    assert.equal(restoredCount(run.log), 0, `${label} manifest must not be restored`);
    assert.equal(recompiledCount(run.log), 1, `${label} manifest must fall back to compiling`);
    assert.ok(run.state.cache.has(file), `${label} manifest must still yield compiled output`);
  }

  // An entry whose module is not a CompiledModule is dropped on its own.
  const manifest = JSON.parse(good) as { entries: Record<string, unknown> };
  manifest.entries[file] = { hash: hashPrecompileSource(BASE_SOURCE), module: { code: 42 } };
  fs.writeFileSync(manifestPath, JSON.stringify(manifest));
  const malformed = await coldRun(root);
  assert.equal(restoredCount(malformed.log), 0, "a malformed entry must not be restored");
  assert.equal(recompiledCount(malformed.log), 1);
}

// ---------------------------------------------------------------------------
// Deleted files are pruned rather than accumulating forever.
// ---------------------------------------------------------------------------
{
  const { root, file } = makeProject(BASE_SOURCE);
  const other = path.join(path.dirname(file), "Other.vue");
  fs.writeFileSync(other, `<template><span>other</span></template>\n`);
  await coldRun(root);
  const manifestPath = path.join(root, PRECOMPILE_CACHE_DIR, manifestFiles(root)[0]!);
  let entries = (JSON.parse(fs.readFileSync(manifestPath, "utf-8")) as { entries: object }).entries;
  assert.equal(Object.keys(entries).length, 2, "both files must be persisted");

  fs.rmSync(other);
  fs.writeFileSync(file, `${BASE_SOURCE}<!-- touched -->\n`);
  await coldRun(root);
  entries = (JSON.parse(fs.readFileSync(manifestPath, "utf-8")) as { entries: object }).entries;
  assert.deepEqual(Object.keys(entries), [file], "the deleted file must be pruned");
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
