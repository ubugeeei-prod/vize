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

import { classifyDialectCoverage, normalizeDialectCoverage } from "../differential/harness.mjs";

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

test("executed Babel oracle records the case-level JSX dialect witness", () => {
  const corpus = oracle.readCorpus();
  const recorded = oracle.readRecorded();
  const coverage = normalizeDialectCoverage(corpus.dialectCoverage, "babel JSX corpus");
  assert.deepEqual(classifyDialectCoverage(coverage), {
    state: "source-inspected",
    dialects: ["jsx-babel"],
    readiness: "unknown",
    measurement: "provisional",
  });
  assert.deepEqual(coverage, {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "babel-plugin-oracle",
        dialects: ["jsx-babel"],
        selector: "oracle.plugin=@vue/babel-plugin-jsx@2.0.1; per-case jsx/tsx is selected by lang",
      },
    ],
  });
  assert.deepEqual(
    Object.fromEntries(
      ["jsx", "tsx"].map((lang) => [
        lang,
        corpus.cases.filter((entry: { lang: string }) => entry.lang === lang).length,
      ]),
    ),
    { jsx: 98, tsx: 2 },
  );
  assert.deepEqual(
    corpus.cases.map((entry: { id: string }) => entry.id).sort(),
    Object.keys(recorded.cases).sort(),
    "every dialect-labelled input needs an actual Babel adapter result",
  );
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

// The plugin's `v` + uppercase directive spelling includes the built-in slots
// directive. Compare whole independently generated modules with the existing
// recorded hyphenated controls; do not derive expected output from Vize.
test("vSlots is the exact camelCase alias of the recorded slots built-in", async () => {
  const [{ default: babel }, { default: plugin }, { default: tsSyntaxPlugin }] = await Promise.all([
    import("@babel/core"),
    import("@vue/babel-plugin-jsx"),
    import("@babel/plugin-syntax-typescript"),
  ]);
  const corpus = oracle.readCorpus();
  const recorded = oracle.readRecorded();
  const ids = [
    "slots/v_slots_only",
    "slots/v_slots_with_children",
    "slots/v_slots_object_literal",
    "slots/v_slots_object_with_children",
    "optimize/v_slots_stability",
  ];
  for (const id of ids) {
    const entry = corpus.cases.find((entry: { id: string }) => entry.id === id);
    assert.ok(entry, id);
    const camel = { ...entry, source: entry.source.replaceAll("v-slots", "vSlots") };
    assert.notEqual(camel.source, entry.source, id);
    const actual = await oracle.transformCase(babel, plugin, tsSyntaxPlugin, camel);
    assert.deepEqual(actual, recorded.cases[id], id);
  }
});
