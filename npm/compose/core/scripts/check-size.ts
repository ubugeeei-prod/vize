import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";

interface PackageExport {
  readonly import: string;
}

interface PackageManifest {
  readonly exports: Readonly<Record<string, PackageExport>>;
}

interface CatalogEntry {
  readonly subpath: string;
  readonly gzipBudgetBytes: number;
}

interface ComposableCatalog {
  readonly entries: readonly CatalogEntry[];
  readonly rootEntry: {
    readonly reexportedEntries: readonly string[];
  };
}

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

const isUnknownArray = (value: unknown): value is readonly unknown[] => Array.isArray(value);

const readString = (value: unknown, description: string): string => {
  assert.ok(typeof value === "string", `${description} must be a string`);
  return value;
};

const readPositiveSafeInteger = (value: unknown, description: string): number => {
  assert.ok(
    typeof value === "number" && Number.isSafeInteger(value) && value > 0,
    `${description} must be a positive safe integer`,
  );
  return value;
};

const readPackageManifest = (value: unknown): PackageManifest => {
  assert.ok(isRecord(value), "package manifest must be an object");
  assert.ok(isRecord(value.exports), "package manifest exports must be an object");

  const exports: Record<string, PackageExport> = {};
  for (const [subpath, conditions] of Object.entries(value.exports)) {
    assert.ok(isRecord(conditions), `package export ${subpath} must be an object`);
    exports[subpath] = {
      import: readString(conditions.import, `package export ${subpath} import`),
    };
  }
  return { exports };
};

const readStringArray = (value: unknown, description: string): readonly string[] => {
  assert.ok(isUnknownArray(value), `${description} must be an array`);
  return value.map((entry, index) => readString(entry, `${description}[${index}]`));
};

const readCatalogEntry = (value: unknown, index: number): CatalogEntry => {
  assert.ok(isRecord(value), `catalog entries[${index}] must be an object`);
  return {
    subpath: readString(value.subpath, `catalog entries[${index}].subpath`),
    gzipBudgetBytes: readPositiveSafeInteger(
      value.gzipBudgetBytes,
      `catalog entries[${index}].gzipBudgetBytes`,
    ),
  };
};

const readComposableCatalog = (value: unknown): ComposableCatalog => {
  assert.ok(isRecord(value), "catalog module must be an object");
  assert.ok(isRecord(value.COMPOSABLE_CATALOG), "catalog module must export COMPOSABLE_CATALOG");
  const catalog = value.COMPOSABLE_CATALOG;
  assert.ok(isUnknownArray(catalog.entries), "catalog entries must be an array");
  assert.ok(isRecord(catalog.rootEntry), "catalog rootEntry must be an object");
  return {
    entries: catalog.entries.map(readCatalogEntry),
    rootEntry: {
      reexportedEntries: readStringArray(
        catalog.rootEntry.reexportedEntries,
        "catalog rootEntry.reexportedEntries",
      ),
    },
  };
};

const packageManifestSource: unknown = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
);
const packageManifest = readPackageManifest(packageManifestSource);
const catalogModule: unknown = await import(new URL("../dist/catalog.mjs", import.meta.url).href);
const catalog = readComposableCatalog(catalogModule);

const subpathBudgets = new Map<string, number>();
for (const entry of catalog.entries) {
  const packageExport = packageManifest.exports[entry.subpath];
  if (packageExport === undefined) {
    console.error(
      JSON.stringify({
        error: "VIZE_COMPOSE_SIZE_BUDGET_UNKNOWN_SUBPATH",
        catalogSubpaths: catalog.entries.map((candidate) => candidate.subpath),
      }),
    );
    process.exit(1);
  }
  subpathBudgets.set(packageExport.import.replace("./dist/", ""), entry.gzipBudgetBytes);
}

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

const sourceCache = new Map<string, string>();
async function readRuntimeSource(file: string): Promise<string> {
  const cached = sourceCache.get(file);
  if (cached !== undefined) return cached;
  const source = await readFile(new URL(`../dist/${file}`, import.meta.url), "utf8");
  sourceCache.set(file, source);
  return source;
}

async function runtimeClosure(entry: string): Promise<Set<string>> {
  const files = new Set<string>();
  const visit = async (file: string): Promise<void> => {
    if (files.has(file)) return;
    files.add(file);
    const source = await readRuntimeSource(file);
    const imports = source.matchAll(/(?:\bfrom\s*|\bimport\s*)["']\.\/([^"']+\.mjs)["']/g);
    for (const match of imports) {
      const importedFile = match[1];
      assert.ok(importedFile);
      await visit(importedFile);
    }
  };
  await visit(entry);
  return files;
}

async function compressedClosureSize(
  entry: string,
): Promise<{ readonly files: readonly string[]; readonly gzipBytes: number }> {
  const files = await runtimeClosure(entry);
  const source = (await Promise.all([...files].map(readRuntimeSource))).join("\n");
  return { files: [...files], gzipBytes: gzipSync(source).byteLength };
}

// The convenience root transitively loads its re-exported runtime entries
// when consumed without a bundler. Measure the complete emitted module graph
// rather than only Rollup's tiny facade.
const rootBudget = catalog.rootEntry.reexportedEntries.reduce((sum, subpath) => {
  const entry = catalog.entries.find((candidate) => candidate.subpath === subpath);
  if (entry === undefined) {
    console.error(
      JSON.stringify({ error: "VIZE_COMPOSE_SIZE_BUDGET_UNKNOWN_ROOT_ENTRY", subpath }),
    );
    process.exitCode = 1;
    return sum;
  }
  return sum + entry.gzipBudgetBytes;
}, 0);
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
