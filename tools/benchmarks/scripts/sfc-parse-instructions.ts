import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { loadRegistry, validateMeasurement } from "./instruction-counts-lib.mjs";
import { verifyRepositoryInstructionSource } from "./instruction-methodology-transition.mjs";

const root = process.cwd();
const file = process.argv[2];
assert.ok(file, "pass the SFC instruction measurement.json");
const registry = loadRegistry(path.join(root, "docs/davinci/plan/sfc-instruction-registry.toml"));
const report = validateMeasurement(JSON.parse(fs.readFileSync(file, "utf8")), registry);
verifyRepositoryInstructionSource(report, root);
for (const run of report.runs) {
  const native = run.sfc_parse_template_only;
  const retained = run.sfc_parse_template_only_retained;
  assert.equal(native.fixture, retained.fixture);
  assert.equal(native.fixture_sha256, retained.fixture_sha256);
  assert.equal(native.window, "routine-return");
  assert.equal(retained.window, native.window);
  console.log(
    `template-only instructions: native=${native.instructions}, retained=${retained.instructions}`,
  );
  assert.ok(
    native.instructions <= retained.instructions,
    "native template-only parsing regressed beyond the retained parser",
  );
}
