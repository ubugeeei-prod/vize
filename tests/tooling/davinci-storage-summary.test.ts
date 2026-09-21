// The storage aggregates (retained alloc Vec totals, per-category and
// per-scope counts) are derived from the reviewed per-file ledger into the
// generated storage summary; the per-file ratchet itself lives in
// davinci-storage-policy.test.ts.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  categoryReasons,
  parseStorageInventory,
  summarizeAllocVecCategories,
  summarizeKind,
  summarizeScopes,
} from "./davinci-storage-inventory.ts";
import { storageKinds } from "./davinci-storage-scan.ts";
import {
  STORAGE_INVENTORY_REL,
  STORAGE_SUMMARY_REGEN,
  STORAGE_SUMMARY_REL,
  renderStorageSummary,
  storageTypeNames,
} from "./davinci-storage-summary.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const expectedRows = parseStorageInventory(
  fs.readFileSync(path.join(repoRoot, STORAGE_INVENTORY_REL), "utf8"),
);

test("the storage aggregates are derived from the exact inventory, never hand-copied", () => {
  const summary = fs.readFileSync(path.join(repoRoot, STORAGE_SUMMARY_REL), "utf8");
  assert.equal(
    summary,
    renderStorageSummary(expectedRows),
    `${STORAGE_SUMMARY_REL} is stale. Regenerate it with:\n  ${STORAGE_SUMMARY_REGEN}`,
  );
  // A single moved ledger count must change the derived page (the byte
  // comparison above cannot pass vacuously).
  const [first, ...rest] = expectedRows;
  const moved = {
    ...first,
    storage: {
      ...first.storage,
      s0String: { ...first.storage.s0String, boundUses: first.storage.s0String.boundUses + 1 },
    },
  };
  assert.notEqual(renderStorageSummary([moved, ...rest]), summary);
  const plan = fs.readFileSync(
    path.join(repoRoot, "davinci-road/plan/storage-boundary.md"),
    "utf8",
  );
  // The plan keeps the policy and the category reasons; every aggregate lives
  // only in the generated summary so no second copy can drift or conflict.
  assert.equal(plan.includes("](./storage-summary.md)"), true);
  assert.doesNotMatch(plan, /^\|\s*(?:contract|analysis|lower|pass|emit)\s*\|\s*\d/mu);
  assert.doesNotMatch(plan, /^\|\s*(?:infra|s1|s2|s3|s1_to_s2|s2_to_s3)\s*\|/mu);
  assert.doesNotMatch(plan, /contain \d+ production files/u);
  for (const category of Object.keys(categoryReasons)) {
    assert.match(plan, new RegExp(`^\\| ${category} +\\| [A-Z]`, "mu"));
  }

  const allocVec = summarizeKind(expectedRows, "allocVec");
  const headline = summary.match(
    /contain (\d+) production files,\s+(\d+) direct[^,]+, and (\d+) bound/iu,
  );
  assert.deepEqual(headline?.slice(1).map(Number), [
    allocVec.files,
    allocVec.directPaths,
    allocVec.boundUses,
  ]);
  const categories = summarizeAllocVecCategories(expectedRows);
  const categoryRows = tableRows(summary, /^(contract|analysis|lower|pass|emit)$/u);
  assert.deepEqual(categoryRows, categories);
  const categoryTotal = Object.values(categories).reduce(
    (sum, row) => ({
      files: sum.files + row.files,
      directPaths: sum.directPaths + row.directPaths,
      boundUses: sum.boundUses + row.boundUses,
    }),
    { files: 0, directPaths: 0, boundUses: 0 },
  );
  assert.deepEqual(categoryTotal, allocVec);

  const scopeSummary = summarizeScopes(expectedRows);
  const scopeRows = new Map(
    [
      ...summary.matchAll(
        /^\|\s*(infra|s1|s2|s3|s1_to_s2|s2_to_s3)\s*\|\s*`([^`]+)`\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|/gmu,
      ),
    ].map(([, scope, type, files, directPaths, boundUses]) => [
      `${scope}:${type}`,
      { files: Number(files), directPaths: Number(directPaths), boundUses: Number(boundUses) },
    ]),
  );
  for (const [scope, kinds] of Object.entries(scopeSummary)) {
    for (const kind of storageKinds) {
      assert.deepEqual(scopeRows.get(`${scope}:${storageTypeNames[kind]}`), kinds[kind]);
    }
  }
  assert.equal(scopeRows.size, 30);
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
