import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  categoryReasons,
  expectedProductionAllocVec,
  parseStorageInventory,
  summarizeAllocVecCategories,
  summarizeKind,
  summarizeScopes,
  type InventoryRow,
  type StorageScope,
} from "./davinci-storage-inventory.ts";
import {
  hasStorage,
  scanStorage,
  storageKinds,
  type FileStorage,
  type StorageKind,
} from "./davinci-storage-scan.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const libraryRoots = [
  "crates/vize_davinci/src",
  "crates/vize_sinopia/src",
  "crates/vize_disegno/src",
  "crates/vize_ricalco/src",
];
const inventoryPath = path.join(repoRoot, "davinci-road/plan/storage-inventory.tsv");
const davinciOptRoot = "crates/vize_davinci/src/bin/davinci-opt/";

function rustFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const absolute = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...rustFiles(absolute));
    else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(absolute);
  }
  return files;
}

function isDavinciOptHostEdge(relative: string): boolean {
  return relative.startsWith(davinciOptRoot);
}

function scopeFor(file: string): StorageScope {
  if (file.startsWith("crates/vize_davinci/")) return "infra";
  if (file.startsWith("crates/vize_sinopia/")) return "s1";
  if (file.startsWith("crates/vize_disegno/")) return "s2";
  if (file.startsWith("crates/vize_ricalco/")) return "s1_to_s2";
  throw new Error(`unknown storage scope: ${file}`);
}

const sources = libraryRoots.flatMap((root) => rustFiles(path.join(repoRoot, root)));
const inventorySource = fs.readFileSync(inventoryPath, "utf8");
const expectedRows = parseStorageInventory(inventorySource);

function measureInventory(): { storage: Map<string, FileStorage>; issues: string[] } {
  const storage = new Map<string, FileStorage>();
  const issues: string[] = [];
  for (const file of sources) {
    const relative = path.relative(repoRoot, file);
    if (isDavinciOptHostEdge(relative)) continue;
    const scanned = scanStorage(fs.readFileSync(file, "utf8"));
    issues.push(...scanned.issues.map((issue) => `${relative}: ${issue}`));
    if (hasStorage(scanned.storage)) storage.set(relative, scanned.storage);
  }
  return { storage, issues };
}

function inventoryViolations(
  measured: ReadonlyMap<string, FileStorage>,
  rows: readonly InventoryRow[],
): string[] {
  const expected = new Map(rows.map((row) => [row.file, row]));
  const violations: string[] = [];
  for (const [file, actual] of measured) {
    const row = expected.get(file);
    if (!row) {
      violations.push(`${file}: unreviewed storage`);
      continue;
    }
    if (row.scope !== scopeFor(file)) violations.push(`${file}: scope ${row.scope} is incorrect`);
    for (const kind of storageKinds) {
      if (!deepMeasurementEqual(actual[kind], row.storage[kind])) {
        violations.push(
          `${file}: ${kind} actual=${format(actual[kind])} inventory=${format(row.storage[kind])}`,
        );
      }
    }
  }
  for (const row of rows) {
    if (!measured.has(row.file)) violations.push(`${row.file}: stale inventory row`);
    if (row.category) assert.ok(categoryReasons[row.category].length > 0);
  }
  return violations;
}

function deepMeasurementEqual(
  left: { directPaths: number; boundUses: number },
  right: { directPaths: number; boundUses: number },
): boolean {
  return left.directPaths === right.directPaths && left.boundUses === right.boundUses;
}

function format(value: { directPaths: number; boundUses: number }): string {
  return `${value.directPaths}/${value.boundUses}`;
}

test("stage storage has no opaque imports or std paths", () => {
  assert.deepEqual(measureInventory().issues, []);
});

test("all owned storage equals the reviewed per-file inventory", () => {
  const measured = measureInventory();
  assert.deepEqual(
    inventoryViolations(measured.storage, expectedRows),
    [],
    "update storage-inventory.tsv only after reviewing every changed count",
  );
  assert.deepEqual(summarizeKind(expectedRows, "allocVec"), expectedProductionAllocVec);
  assert.deepEqual(summarizeKind(expectedRows, "allocString"), {
    files: 0,
    directPaths: 0,
    boundUses: 0,
  });
});

test("production inventory excludes cfg(test) size evidence", () => {
  const byFile = new Map(expectedRows.map((row) => [row.file, row]));
  assert.deepEqual(byFile.get("crates/vize_ricalco/src/emit/on.rs")?.storage.allocVec, {
    directPaths: 0,
    boundUses: 0,
  });
  assert.deepEqual(byFile.get("crates/vize_davinci/src/side_table.rs")?.storage.allocVec, {
    directPaths: 1,
    boundUses: 2,
  });
});

test("alloc Vec categories cover module-bound uses without a direct path", () => {
  const header = inventorySource.split("\n", 1)[0];
  const uncategorized = `${header}\ninfra\t-\tfixture.rs\t0\t1\t0\t0\t0\t0\t0\t0\n`;
  assert.throws(() => parseStorageInventory(uncategorized), /alloc Vec category mismatch/u);
});

test("the plan summaries are generated from the exact inventory", () => {
  const plan = fs.readFileSync(
    path.join(repoRoot, "davinci-road/plan/storage-boundary.md"),
    "utf8",
  );
  const headline = plan.match(
    /contain (\d+) production files,\s+(\d+) direct[^,]+, and (\d+) bound/iu,
  );
  assert.deepEqual(headline?.slice(1).map(Number), [
    expectedProductionAllocVec.files,
    expectedProductionAllocVec.directPaths,
    expectedProductionAllocVec.boundUses,
  ]);
  const categoryRows = tableRows(plan, /^(contract|analysis|lower|pass|emit)$/u);
  assert.deepEqual(categoryRows, summarizeAllocVecCategories(expectedRows));

  const scopeSummary = summarizeScopes(expectedRows);
  const scopeRows = new Map(
    [
      ...plan.matchAll(
        /^\|\s*(infra|s1|s2|s1_to_s2)\s*\|\s*`([^`]+)`\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|/gmu,
      ),
    ].map(([, scope, type, files, directPaths, boundUses]) => [
      `${scope}:${type}`,
      { files: Number(files), directPaths: Number(directPaths), boundUses: Number(boundUses) },
    ]),
  );
  const names: Record<StorageKind, string> = {
    allocVec: "alloc::vec::Vec",
    allocString: "alloc::string::String",
    s0String: "vize_s0::String",
    arenaVec: "vize_s0::Vec",
    smallVec: "vize_s0::SmallVec",
  };
  for (const [scope, kinds] of Object.entries(scopeSummary)) {
    for (const kind of storageKinds) {
      assert.deepEqual(scopeRows.get(`${scope}:${names[kind]}`), kinds[kind]);
    }
  }
  assert.equal(scopeRows.size, 20);
});

function tableRows(source: string, keyPattern: RegExp): Record<string, unknown> {
  return Object.fromEntries(
    [...source.matchAll(/^\|\s*([^|]+?)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|/gmu)]
      .filter(([, key]) => keyPattern.test(key))
      .map(([, key, files, directPaths, boundUses]) => [
        key,
        { files: Number(files), directPaths: Number(directPaths), boundUses: Number(boundUses) },
      ]),
  );
}

test("scanner resolves root, self, group, module, and raw aliases", () => {
  const cases: Array<[string, number, number]> = [
    ["use ::alloc::vec::Vec as Heap; type T = Heap<u8>;", 1, 1],
    ["use crate::alloc::vec::Vec as r#Heap; type T = r#Heap<u8>;", 1, 1],
    ["use self::alloc::vec as heap; type T = heap::Vec<u8>;", 0, 1],
    ["use {alloc::vec as heap}; type T = heap::Vec<u8>;", 0, 1],
    ["use alloc::{self as heap}; type T = heap::vec::Vec<u8>;", 0, 1],
    ["use alloc::vec::{self}; type T = vec::Vec<u8>;", 0, 1],
    ["use {alloc::{vec::{self}}}; type T = vec::Vec<u8>;", 0, 1],
    ["extern crate alloc as heap; type T = heap::vec::Vec<u8>;", 0, 1],
    [
      "use alloc::vec::{self as heap, Vec as r#Items}; type A=heap::Vec<u8>; type B=r#Items<u8>;",
      1,
      2,
    ],
  ];
  for (const [source, directPaths, boundUses] of cases) {
    const result = scanStorage(source);
    assert.deepEqual(result.issues, []);
    assert.deepEqual(result.storage.allocVec, { directPaths, boundUses });
  }
});

test("scanner rejects escape hatches and masks only cfg(test) items", () => {
  assert.notEqual(scanStorage("use alloc::*;").issues.length, 0);
  assert.notEqual(scanStorage("use {std::vec as heap}; type T=heap::Vec<u8>;").issues.length, 0);
  assert.deepEqual(
    scanStorage("use alloc::{string as text}; type T=text::String;").storage.allocString,
    { directPaths: 0, boundUses: 1 },
  );
  assert.deepEqual(
    scanStorage("use alloc::string::{self}; type T=string::String;").storage.allocString,
    { directPaths: 0, boundUses: 1 },
  );
  assert.deepEqual(
    scanStorage("#[cfg(test)] mod tests { use alloc::vec::Vec; type T=Vec<u8>; }").storage.allocVec,
    { directPaths: 0, boundUses: 0 },
  );
  assert.deepEqual(
    scanStorage("#[cfg(not(test))] use alloc::vec::Vec; type T=Vec<u8>;").storage.allocVec,
    { directPaths: 1, boundUses: 1 },
  );
  const arrayStatic = `
    #[cfg(test)]
    static TEST_STORAGE: [Option<alloc::vec::Vec<u8>>; 2] = [None, None];
    use alloc::vec::Vec as Prod;
    type Production = Prod<u8>;
  `;
  assert.deepEqual(scanStorage(arrayStatic).storage.allocVec, { directPaths: 1, boundUses: 1 });
  const blockConst = `
    #[cfg(test)]
    const TEST_STORAGE: usize = { let _: Option<alloc::vec::Vec<u8>> = None; 1 };
    use alloc::vec::Vec as Prod;
    type Production = Prod<u8>;
  `;
  assert.deepEqual(scanStorage(blockConst).storage.allocVec, { directPaths: 1, boundUses: 1 });
  const qualifiedFunctions = [
    "const unsafe fn",
    "pub(crate) async unsafe fn",
    'pub(crate) unsafe extern "C" fn',
    'pub const unsafe extern r#"C"# fn',
  ];
  for (const qualifiers of qualifiedFunctions) {
    const qualifiedFunction = `
      #[cfg(test)] ${qualifiers} helper() {}
      use alloc::vec::Vec as Prod;
      type Production = Prod<u8>;
    `;
    assert.deepEqual(scanStorage(qualifiedFunction).storage.allocVec, {
      directPaths: 1,
      boundUses: 1,
    });
  }
  const constGenericFunctions = [
    "fn helper() -> Marker<{ 1 }>",
    "fn helper<T>() where Bound<{ 1 }>: Trait",
    "fn helper() -> Outer<Inner<{ 1 < 2 }>>",
    "fn helper<T>() where Outer<Bound<{ 8 >> 1 }>>: Trait",
  ];
  for (const signature of constGenericFunctions) {
    const constGenericFunction = `
      #[cfg(test)] ${signature} {
        let _: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
        todo!()
      }
      use alloc::vec::Vec as Prod;
      type Production = Prod<u8>;
    `;
    assert.deepEqual(scanStorage(constGenericFunction).storage.allocVec, {
      directPaths: 1,
      boundUses: 1,
    });
  }
  const visibleConstGenericFunction = `
    #[cfg(not(test))]
    fn helper() -> Marker<{ 1 }> {
      let _: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
    }
    use alloc::vec::Vec as Prod;
    type Production = Prod<u8>;
  `;
  assert.deepEqual(scanStorage(visibleConstGenericFunction).storage.allocVec, {
    directPaths: 3,
    boundUses: 1,
  });
  const comparisonInitializer = `
    #[cfg(test)]
    const TEST_COMPARISON: bool = 1 < 2;
    use alloc::vec::Vec as Prod;
    type Production = Prod<u8>;
  `;
  assert.deepEqual(scanStorage(comparisonInitializer).storage.allocVec, {
    directPaths: 1,
    boundUses: 1,
  });
  assert.deepEqual(scanStorage('let text = "alloc::vec::Vec"; // use alloc::*').storage.allocVec, {
    directPaths: 0,
    boundUses: 0,
  });
});

test("davinci-opt is the exact host edge", () => {
  assert.equal(isDavinciOptHostEdge("crates/vize_davinci/src/bin/davinci-opt/main.rs"), true);
  assert.equal(isDavinciOptHostEdge("crates/vize_davinci/src/lib.rs"), false);
});
