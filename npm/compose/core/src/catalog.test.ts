import assert from "node:assert/strict";
import { test } from "node:test";

import { COMPOSABLE_CATALOG } from "./catalog.ts";

function sortedUnique(values: readonly string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function includes(values: readonly string[], value: string): boolean {
  return values.includes(value);
}

void test("the catalog is deterministic, serializable data", () => {
  const serialized = JSON.stringify(COMPOSABLE_CATALOG);
  assert.deepEqual(JSON.parse(serialized), COMPOSABLE_CATALOG);
  assert.equal(JSON.stringify(JSON.parse(serialized)), serialized);
  assert.equal(COMPOSABLE_CATALOG.schemaVersion, 1);
  assert.equal(COMPOSABLE_CATALOG.catalogStability, "stable");
  assert.equal(COMPOSABLE_CATALOG.packageInstallation.status, "available");
  assert.equal(COMPOSABLE_CATALOG.sourceInstallation.status, "unavailable");
});

void test("entries have one ordered export subpath, source, and integral size budget", () => {
  const { entries, rootEntry } = COMPOSABLE_CATALOG;
  const subpaths = entries.map((entry) => entry.subpath);

  assert.deepEqual(subpaths, sortedUnique(subpaths));
  assert.deepEqual(
    sortedUnique([...rootEntry.reexportedEntries, ...rootEntry.isolatedEntries]),
    subpaths,
  );
  assert.deepEqual(rootEntry.reexportedEntries, sortedUnique(rootEntry.reexportedEntries));
  assert.deepEqual(rootEntry.isolatedEntries, sortedUnique(rootEntry.isolatedEntries));
  assert.deepEqual(rootEntry.isolatedEntries, ["./temporal"]);
  for (const entry of entries) {
    const basename = entry.subpath.slice(2);
    assert.equal(entry.source, `src/${basename}.ts`);
    assert.ok(Number.isSafeInteger(entry.gzipBudgetBytes));
    assert.ok(entry.gzipBudgetBytes > 0);
    assert.equal(new Set(entry.runtimeExports).size, entry.runtimeExports.length);
    assert.equal(new Set(entry.utilities).size, entry.utilities.length);
    assert.ok(
      entry.utilities.every((utility) => includes(entry.runtimeExports, utility)),
      `${entry.subpath} utilities must be runtime exports`,
    );
  }
});

void test("utility membership is exact and operational metadata is canonical", () => {
  const { entries, targets, utilities } = COMPOSABLE_CATALOG;
  const entryBySubpath = new Map<string, (typeof entries)[number]>(
    entries.map((entry) => [entry.subpath, entry]),
  );
  const utilityByName = new Map<string, (typeof utilities)[number]>(
    utilities.map((utility) => [utility.name, utility]),
  );

  assert.equal(utilityByName.size, utilities.length);
  assert.deepEqual(
    entries.flatMap((entry) => entry.utilities),
    utilities.map((utility) => utility.name),
  );

  for (const utility of utilities) {
    assert.ok(entryBySubpath.has(utility.entry), `${utility.name} references a public entry`);
    assert.ok(
      includes(entryBySubpath.get(utility.entry)?.utilities ?? [], utility.name),
      `${utility.name} is owned by ${utility.entry}`,
    );
    assert.ok(utility.targets.length > 0, `${utility.name} declares at least one target`);
    assert.equal(new Set(utility.targets).size, utility.targets.length);
    assert.deepEqual(
      utility.targets,
      targets.filter((target) => includes(utility.targets, target)),
      `${utility.name} targets use canonical order`,
    );
    assert.ok(utility.cleanupOwners.length > 0);
    assert.equal(new Set(utility.cleanupOwners).size, utility.cleanupOwners.length);
    if (includes(utility.cleanupOwners, "none")) {
      assert.deepEqual(utility.cleanupOwners, ["none"]);
    }
    assert.deepEqual(
      utility.runtimeGlobals,
      sortedUnique(utility.runtimeGlobals),
      `${utility.name} runtime globals are sorted and unique`,
    );
    assert.equal(new Set(utility.dependencies).size, utility.dependencies.length);
    for (const dependency of utility.dependencies) {
      assert.ok(utilityByName.has(dependency), `${utility.name} depends on known ${dependency}`);
      assert.notEqual(dependency, utility.name);
    }
  }
});

void test("utility dependencies form an acyclic graph", () => {
  const utilityByName = new Map<string, (typeof COMPOSABLE_CATALOG.utilities)[number]>(
    COMPOSABLE_CATALOG.utilities.map((utility) => [utility.name, utility]),
  );
  const visiting = new Set<string>();
  const visited = new Set<string>();

  const visit = (name: string, path: readonly string[]): void => {
    if (visited.has(name)) return;
    assert.ok(!visiting.has(name), `dependency cycle: ${[...path, name].join(" -> ")}`);
    visiting.add(name);
    const utility = utilityByName.get(name);
    assert.ok(utility);
    for (const dependency of utility.dependencies) visit(dependency, [...path, name]);
    visiting.delete(name);
    visited.add(name);
  };

  for (const name of utilityByName.keys()) visit(name, []);
  assert.equal(visited.size, COMPOSABLE_CATALOG.utilities.length);
});
