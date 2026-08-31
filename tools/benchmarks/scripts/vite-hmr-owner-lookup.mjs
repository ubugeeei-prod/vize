/**
 * Cost of the owner lookup that runs first on every Vite hot update (#3403).
 *
 * `handleHotUpdateHook` has to know which SFCs pulled a changed file in through
 * `<script src>` / `<template src>` / `<style src>` before it can do anything
 * else, including before the `.vue` fast path. This measures that lookup two
 * ways against the same synthetic caches:
 *
 * - `scan`  — the pre-#3403 implementation: walk both caches, `path.resolve`
 *             every dependency of every cached module.
 * - `index` — the reverse index the caches now maintain.
 *
 * Wall clock on a contended machine is noise, so the primary number is the
 * `path.resolve` call count per hot update, which is deterministic. The index
 * moves that work to insert time, so insert-side resolves are reported too.
 *
 *   node tools/benchmarks/scripts/vite-hmr-owner-lookup.mjs
 *   node tools/benchmarks/scripts/vite-hmr-owner-lookup.mjs --sizes 1000,3000,10000 --iterations 200
 */

import path from "node:path";

import {
  CompiledModuleCache,
  ownersOfDependency,
} from "../npm/builder/vite/src/plugin/compiled-module-cache.ts";

const nativeResolve = path.resolve.bind(path);
let resolveCalls = 0;
path.resolve = (...args) => {
  resolveCalls += 1;
  return nativeResolve(...args);
};

function parseArgs(argv) {
  const options = { sizes: [1000, 3000, 10000], iterations: 200 };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--sizes") options.sizes = argv[++i].split(",").map(Number);
    else if (argv[i] === "--iterations") options.iterations = Number(argv[++i]);
  }
  return options;
}

function buildEntries(size, withSrcImports) {
  const entries = [];
  for (let i = 0; i < size; i += 1) {
    entries.push([
      `/project/src/Component${i}.vue`,
      {
        code: "export default {}",
        dependencies: withSrcImports ? [`/project/src/styles/component${i}.css`] : [],
      },
    ]);
  }
  return entries;
}

/** The pre-#3403 lookup, reproduced exactly. */
function scanOwners(caches, dependencyFile) {
  const normalizedDependency = path.resolve(dependencyFile);
  const owners = new Set();
  for (const cache of caches) {
    for (const [vueFile, compiled] of cache) {
      if (compiled.dependencies?.some((d) => path.resolve(d) === normalizedDependency)) {
        owners.add(vueFile);
      }
    }
  }
  return [...owners];
}

function indexOwners(caches, dependencyFile) {
  const normalizedDependency = path.resolve(dependencyFile);
  const owners = new Set();
  for (const cache of caches) {
    for (const vueFile of ownersOfDependency(cache, normalizedDependency)) owners.add(vueFile);
  }
  return [...owners];
}

function measure(lookup, caches, changedFile, iterations) {
  lookup(caches, changedFile);
  resolveCalls = 0;
  const started = process.hrtime.bigint();
  for (let i = 0; i < iterations; i += 1) lookup(caches, changedFile);
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
  return { msPerCall: elapsedMs / iterations, resolvesPerCall: resolveCalls / iterations };
}

const { sizes, iterations } = parseArgs(process.argv.slice(2));
const rows = [];

for (const size of sizes) {
  for (const withSrcImports of [false, true]) {
    const entries = buildEntries(size, withSrcImports);

    const plain = new Map(entries);
    resolveCalls = 0;
    const indexed = new CompiledModuleCache();
    for (const [file, module] of entries) indexed.set(file, module);
    const insertResolves = resolveCalls;

    // The common case: an ordinary `.vue` save, which owns nothing but still
    // pays for the lookup because it runs before the `.vue` fast path.
    const changedFile = "/project/src/Component0.vue";

    const scan = measure(scanOwners, [plain, new Map()], changedFile, iterations);
    const index = measure(
      indexOwners,
      [indexed, new CompiledModuleCache()],
      changedFile,
      iterations,
    );

    const scanOwnersResult = scanOwners([plain, new Map()], changedFile);
    const indexOwnersResult = indexOwners([indexed, new CompiledModuleCache()], changedFile);
    if (JSON.stringify(scanOwnersResult) !== JSON.stringify(indexOwnersResult)) {
      throw new Error(`owner mismatch at size ${size}`);
    }

    rows.push({
      size,
      srcImports: withSrcImports ? "one per SFC" : "none",
      scanResolves: scan.resolvesPerCall,
      indexResolves: index.resolvesPerCall,
      scanMs: scan.msPerCall,
      indexMs: index.msPerCall,
      insertResolves,
    });
  }
}

console.log(`iterations per measurement: ${iterations}`);
console.log(
  "| cached SFCs | src imports | resolve() per update (scan) | resolve() per update (index) | ms per update (scan) | ms per update (index) | resolve() to fill the index |",
);
console.log("| ---: | --- | ---: | ---: | ---: | ---: | ---: |");
for (const row of rows) {
  console.log(
    `| ${row.size.toLocaleString("en-US")} | ${row.srcImports} | ${row.scanResolves} | ${row.indexResolves} | ${row.scanMs.toFixed(4)} | ${row.indexMs.toFixed(4)} | ${row.insertResolves.toLocaleString("en-US")} |`,
  );
}
