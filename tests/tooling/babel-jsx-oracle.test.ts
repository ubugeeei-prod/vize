/**
 * Guards the `@vue/babel-plugin-jsx` differential oracle used by
 * `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs`.
 *
 * The Rust suite reads a *recorded* copy of babel's output so it can run
 * offline. That recording is only trustworthy if something re-derives it from
 * the real plugin, which is what this test does: it re-runs every corpus case
 * through the installed `@vue/babel-plugin-jsx` and asserts the result equals
 * the committed recording, field for field. A plugin bump therefore shows up
 * here as a hard failure rather than as silent compatibility drift.
 *
 * It also asserts the inventory markdown covers exactly the corpus — repo policy
 * is that an unwritten edge case does not exist, so a corpus row with no
 * inventory row (or vice versa) is a failure.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const compatDir = join(repoRoot, "crates", "vize_atelier_jsx", "tests", "babel_compat");
const oracle = await import(join(compatDir, "oracle.mjs"));

test("recorded babel output matches the installed @vue/babel-plugin-jsx", async () => {
  const recomputed = await oracle.runOracle();
  const recorded = oracle.readRecorded();

  assert.equal(recomputed.pluginVersion, recorded.pluginVersion);
  assert.deepEqual(Object.keys(recomputed.cases).sort(), Object.keys(recorded.cases).sort());
  assert.deepEqual(recomputed.cases, recorded.cases);
});

test("corpus pins the plugin version the recording was produced with", () => {
  const corpus = oracle.readCorpus();
  const recorded = oracle.readRecorded();
  assert.equal(corpus.oracle.plugin, "@vue/babel-plugin-jsx");
  assert.equal(corpus.oracle.pluginVersion, recorded.pluginVersion);
});

test("inventory markdown documents exactly the corpus cases", () => {
  const corpus = oracle.readCorpus();
  const inventory = readFileSync(join(compatDir, "..", "BABEL_COMPAT_INVENTORY.md"), "utf8");

  // Per-case rows are the table rows whose first cell is a backticked
  // `<category>/<name>` id using one of the corpus's own categories. Keying off
  // the category set is what separates them from the document's other tables
  // (which also start with a backticked cell, e.g. artifact paths) without
  // pre-supposing which ids exist — so a missing *or* an extra case row still
  // fails.
  const categories = new Set(corpus.cases.map((entry: { category: string }) => entry.category));
  const documented = new Set<string>();
  for (const line of inventory.split("\n")) {
    if (!line.startsWith("| `")) continue;
    const firstCell = line.slice(1, line.indexOf("|", 1)).trim();
    if (!firstCell.startsWith("`") || !firstCell.endsWith("`")) continue;
    const id = firstCell.slice(1, -1);
    if (categories.has(id.slice(0, id.indexOf("/")))) {
      documented.add(id);
    }
  }

  const expected = corpus.cases.map((entry: { id: string }) => entry.id).sort();
  assert.deepEqual([...documented].sort(), expected);
});
