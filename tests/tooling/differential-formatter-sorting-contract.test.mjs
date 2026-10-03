import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterApiManifest } from "../differential/formatter-api.mjs";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const file = path.join(
  root,
  "tests/_fixtures/differential/formatter-history/import-sorting-manifest.json",
);
const expectedIds = [
  "usercard-enabled",
  "usercard-disabled",
  "usercard-omitted",
  "ordinary-ts",
  "ordinary-tsx",
  "custom-groups",
  "side-effects-comments",
  "descending-internal",
  "newline-partitions",
  "direct-typescript",
  "direct-disabled",
  "invalid-boundary",
  "invalid-order",
  "invalid-true",
].map((id) => "import-sorting/" + id);

void test("sorting history retains separate public APIs and complete option source states", () => {
  const { cases } = loadFormatterApiManifest(file, root);
  assert.deepEqual(
    cases.map(({ id }) => id),
    expectedIds,
  );
  assert.deepEqual(
    cases.map(({ api }) => api),
    [
      ...Array(9).fill("GlyphFormatter::format"),
      "format_script_with_sort_imports",
      "format_script_with_sort_imports",
      "GlyphFormatter::format",
      "format_script_with_sort_imports",
      "format_script_with_sort_imports",
    ],
  );
  assert.deepEqual(
    cases.slice(0, 3).map(({ optionsArgv, effectiveOptions }) => ({
      argv: optionsArgv,
      provided: effectiveOptions.sortImportsProvided,
      setting: effectiveOptions.sortImports,
    })),
    [
      { argv: ["--sort-options-json", "--sort-imports", "{}"], provided: true, setting: {} },
      { argv: ["--sort-options-json", "--sort-imports", "false"], provided: true, setting: false },
      { argv: ["--sort-options-json"], provided: false, setting: null },
    ],
  );
  assert.deepEqual(
    cases.map(({ passCount }) => passCount),
    [...Array(11).fill(3), 1, 1, 1],
  );
  assert.deepEqual(
    cases.map(({ native }) => native),
    Array(14).fill("unsupported"),
  );
  assert.deepEqual(
    cases.slice(11).map(({ typedError }) => typedError),
    Array(3).fill("ScriptFormatError"),
  );
});

void test("sorting history rejects altered API, options, complete probes and source authorities", (t) => {
  const folder = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-sorting-contract-"));
  t.after(() => fs.rmSync(folder, { recursive: true, force: true }));
  for (const mutate of [
    (manifest) => {
      delete manifest.featureIssue;
    },
    (manifest) => {
      delete manifest.cases[0].importSorting;
    },
    (manifest) => {
      manifest.cases[0].api = "format_sfc";
    },
    (manifest) => {
      manifest.cases[0].importSorting.setting = false;
    },
    (manifest) => {
      manifest.cases[2].importSorting.provided = true;
    },
    (manifest) => {
      manifest.cases[0].importSorting.resolvedOptions = "Ok(None)";
    },
    (manifest) => {
      manifest.cases[0].importSorting.unregistered = true;
    },
    (manifest) => {
      manifest.cases[0].sourceOptionsProbe.sha256 = "0".repeat(64);
    },
    (manifest) => {
      manifest.cases[0].witness.sourceSha256 = "0".repeat(64);
    },
    (manifest) => {
      manifest.cases[0].witness.function = "invented_option_law";
    },
    (manifest) => {
      manifest.cases[11].typedError = "IoError";
    },
    (manifest) => {
      manifest.nativeHandled = 1;
    },
  ]) {
    const manifest = JSON.parse(fs.readFileSync(file, "utf8"));
    mutate(manifest);
    const altered = path.join(folder, "altered.json");
    fs.writeFileSync(altered, JSON.stringify(manifest));
    assert.throws(() => loadFormatterApiManifest(altered, root));
  }
});

void test("sorting options reuse the real merge execution and keep pure contracts in PR selection", () => {
  const execution = "tests/tooling/differential-formatter-api-execution.test.mjs";
  const contract = "tests/tooling/differential-formatter-sorting-contract.test.mjs";
  const changed = ["crates/vize_glyph/examples/formatter_observe.rs"];
  const ordinary = planToolingTests(changed, { cwd: root });
  assert(ordinary.tests.includes(contract));
  assert(!ordinary.tests.includes(execution));
  const protectedPlan = planToolingTests(changed, { cwd: root, tier: "merge" });
  assert(protectedPlan.tests.includes(execution));
  assert(protectedPlan.tests.includes(contract));
});
