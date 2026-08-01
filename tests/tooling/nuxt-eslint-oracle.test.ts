/**
 * Guards the `@nuxt/eslint` differential oracle used by
 * `npm/framework/nuxt-lint-config/src/oracle.test.ts`.
 *
 * That suite reads a *recorded* copy of `@nuxt/eslint`'s output so the package
 * tests run offline. The recording is only trustworthy if something re-derives
 * it from the real packages, which is what this test does: it re-runs every
 * corpus case through the installed `@nuxt/eslint` and `@nuxt/eslint-config`
 * and asserts the result equals the committed recording, field for field. A
 * dependency bump therefore shows up here as a hard failure rather than as
 * silent compatibility drift.
 *
 * It also asserts the inventory markdown covers exactly the corpus — repo
 * policy is that an unwritten edge case does not exist, so a corpus row with no
 * inventory row (or vice versa) is a failure.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const compatDir = join(
  repoRoot,
  "npm",
  "framework",
  "nuxt-lint-config",
  "test",
  "nuxt-eslint-compat",
);
const oracle = await import(join(compatDir, "oracle.mjs"));

test("recorded output matches the installed @nuxt/eslint packages", async () => {
  const recomputed = await oracle.runOracle();
  const recorded = oracle.readRecorded();

  assert.equal(recomputed.schemaVersion, 6);
  assert.equal(recorded.schemaVersion, 6);
  assert.equal(recomputed.moduleVersion, recorded.moduleVersion);
  assert.equal(recomputed.configVersion, recorded.configVersion);
  assert.equal(recomputed.pluginVersion, recorded.pluginVersion);
  assert.equal(recomputed.typeScriptDetected, recorded.typeScriptDetected);
  assert.deepEqual(recomputed.importGlobals, recorded.importGlobals);
  assert.deepEqual(recomputed.checkerOptions, recorded.checkerOptions);
  assert.deepEqual(Object.keys(recomputed.cases).sort(), Object.keys(recorded.cases).sort());
  assert.deepEqual(recomputed.dirDefaults, recorded.dirDefaults);
  assert.deepEqual(recomputed.preferImportMetaCases, recorded.preferImportMetaCases);
  assert.deepEqual(recomputed.noPageMetaRuntimeValuesCases, recorded.noPageMetaRuntimeValuesCases);
  assert.deepEqual(recomputed.noNuxtConfigTestKeyCases, recorded.noNuxtConfigTestKeyCases);
  assert.deepEqual(recomputed.cases, recorded.cases);
});

test("corpus pins the package versions the recording was produced with", () => {
  const corpus = oracle.readCorpus();
  const recorded = oracle.readRecorded();
  assert.equal(corpus.oracle.module, "@nuxt/eslint");
  assert.equal(corpus.oracle.config, "@nuxt/eslint-config");
  assert.equal(corpus.oracle.moduleVersion, recorded.moduleVersion);
  assert.equal(corpus.oracle.configVersion, recorded.configVersion);
  assert.equal(corpus.oracle.plugin, "@nuxt/eslint-plugin");
  assert.equal(corpus.oracle.pluginVersion, recorded.pluginVersion);
});

test("inventory markdown documents exactly the corpus cases", () => {
  const corpus = oracle.readCorpus();
  const inventory = readFileSync(join(compatDir, "INVENTORY.md"), "utf8");

  // Per-case rows are the table rows whose first cell is a backticked id using
  // one of the corpus's own prefixes. Keying off the prefix set is what
  // separates them from the document's other tables (which also start with a
  // backticked cell, e.g. config item names) without pre-supposing which ids
  // exist — so a missing *or* an extra case row still fails.
  const ids = [
    corpus.importGlobals.id,
    ...corpus.checkerCases.map((entry: { id: string }) => entry.id),
    ...corpus.preferImportMetaCases.map((entry: { id: string }) => entry.id),
    ...corpus.noPageMetaRuntimeValuesCases.map((entry: { id: string }) => entry.id),
    ...corpus.noNuxtConfigTestKeyCases.map((entry: { id: string }) => entry.id),
    ...corpus.dirDefaultCases.map((entry: { id: string }) => entry.id),
    ...corpus.cases.map((entry: { id: string }) => entry.id),
  ];
  const prefixes = new Set(ids.map((id) => id.slice(0, id.indexOf("/"))));

  const documented = new Set<string>();
  for (const line of inventory.split("\n")) {
    if (!line.startsWith("| `")) continue;
    const firstCell = line.slice(1, line.indexOf("|", 1)).trim();
    if (!firstCell.startsWith("`") || !firstCell.endsWith("`")) continue;
    const id = firstCell.slice(1, -1);
    if (prefixes.has(id.slice(0, id.indexOf("/")))) {
      documented.add(id);
    }
  }

  const byId = (left: string, right: string) => left.localeCompare(right);
  assert.deepEqual([...documented].sort(byId), [...ids].sort(byId));
});
