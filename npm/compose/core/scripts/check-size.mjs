import { readFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";

const kibibyte = 1024;

/** Per-subpath compressed budgets; every public runtime entry is mandatory. */
const subpathBudgets = new Map([
  ["abort-signal.mjs", 3 * kibibyte],
  ["async-resource.mjs", 2 * kibibyte],
  ["capability.mjs", 1 * kibibyte],
  ["disposal-scope.mjs", 2 * kibibyte],
  ["event-listener.mjs", 2 * kibibyte],
  ["locale.mjs", 2 * kibibyte],
  ["media-query.mjs", 1 * kibibyte],
  ["scope.mjs", 0.5 * kibibyte],
  ["temporal.mjs", 2 * kibibyte],
  ["timeout-scheduler.mjs", 0.25 * kibibyte],
  ["use-counter.mjs", 1.5 * kibibyte],
  ["use-debounced.mjs", 2 * kibibyte],
  ["use-history.mjs", 2.5 * kibibyte],
  ["use-previous.mjs", 1 * kibibyte],
  ["use-throttled.mjs", 2.5 * kibibyte],
  ["use-toggle.mjs", 0.5 * kibibyte],
]);

const packageManifest = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
);
const exportedEntries = new Map(
  Object.entries(packageManifest.exports).map(([subpath, conditions]) => [
    conditions.import.replace("./dist/", ""),
    subpath,
  ]),
);

const expectedBudgetFiles = [...exportedEntries.keys()]
  .filter((file) => file !== "index.mjs")
  .sort((left, right) => left.localeCompare(right));
const actualBudgetFiles = [...subpathBudgets.keys()].sort((left, right) =>
  left.localeCompare(right),
);
if (JSON.stringify(actualBudgetFiles) !== JSON.stringify(expectedBudgetFiles)) {
  console.error(
    JSON.stringify({
      error: "VIZE_COMPOSE_SIZE_BUDGET_EXPORT_MISMATCH",
      expectedBudgetFiles,
      actualBudgetFiles,
    }),
  );
  process.exit(1);
}

const sourceCache = new Map();
async function readRuntimeSource(file) {
  const cached = sourceCache.get(file);
  if (cached !== undefined) return cached;
  const source = await readFile(new URL(`../dist/${file}`, import.meta.url), "utf8");
  sourceCache.set(file, source);
  return source;
}

async function runtimeClosure(entry) {
  const files = new Set();
  const visit = async (file) => {
    if (files.has(file)) return;
    files.add(file);
    const source = await readRuntimeSource(file);
    const imports = source.matchAll(/(?:\bfrom\s*|\bimport\s*)["']\.\/([^"']+\.mjs)["']/g);
    for (const match of imports) await visit(match[1]);
  };
  await visit(entry);
  return files;
}

async function compressedClosureSize(entry) {
  const files = await runtimeClosure(entry);
  const source = (await Promise.all([...files].map(readRuntimeSource))).join("\n");
  return { files: [...files], gzipBytes: gzipSync(source).byteLength };
}

// The convenience root transitively loads its re-exported runtime entries
// when consumed without a bundler. Measure the complete emitted module graph
// rather than only Rollup's tiny facade.
const rootBudget = [...subpathBudgets.values()].reduce((sum, budget) => sum + budget, 0);
const rootSize = await compressedClosureSize("index.mjs");
console.log(
  JSON.stringify({
    entry: "@vizejs/composable",
    gzipBytes: rootSize.gzipBytes,
    maximumGzipBytes: rootBudget,
    files: rootSize.files,
  }),
);
if (rootSize.gzipBytes > rootBudget) process.exitCode = 1;

for (const [file, maximumGzipBytes] of subpathBudgets) {
  const size = await compressedClosureSize(file);
  const subpath = exportedEntries.get(file);
  if (subpath === undefined) {
    console.error(JSON.stringify({ error: "VIZE_COMPOSE_SIZE_BUDGET_UNKNOWN_ENTRY", file }));
    process.exitCode = 1;
    continue;
  }

  console.log(
    JSON.stringify({
      entry: `@vizejs/composable${subpath === "." ? "" : subpath.slice(1)}`,
      gzipBytes: size.gzipBytes,
      maximumGzipBytes,
      files: size.files,
    }),
  );
  if (size.gzipBytes > maximumGzipBytes) process.exitCode = 1;
}
