import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../tools/support/compat/fixtures/fixture-compatibility-ledger.mjs";
import { createCompatibilityReport } from "../../tools/support/compat/fixtures/fixture-compatibility-report.mjs";

const context = createCompatibilityContext();
const ledger = readCompatibilityLedger();

test("pinned project dialect presence stays distinct from per-file coverage", () => {
  validateCompatibilityLedger(ledger, context);
  const coverage = createCompatibilityReport(ledger, context).dialectCoverage;
  assert.equal(coverage.unknownFixtureCount, 132);
  assert.equal(coverage.partialFixtureCount, 15);
  assert.deepEqual(coverage.presentInFixtures["vue2-sfc"], [
    "tests/_fixtures/_git/bootstrap-vue",
    "tests/_fixtures/_git/cube-ui",
    "tests/_fixtures/_git/element",
    "tests/_fixtures/_git/mobile-web-best-practice",
    "tests/_fixtures/_git/vue-element-admin",
    "tests/_fixtures/_git/vue-select",
    "tests/_fixtures/_git/vue2-elm",
    "tests/_fixtures/_git/vux",
  ]);
  assert.deepEqual(coverage.presentInFixtures["vue2.7-sfc"], ["tests/_fixtures/_git/vue-material"]);
  assert.deepEqual(coverage.presentInFixtures["petite-vue"], [
    "tests/_fixtures/_git/petite-vue",
    "tests/_fixtures/_git/wakapi",
  ]);
  assert.deepEqual(coverage.presentInFixtures["pug-template"], [
    "tests/_fixtures/_git/dho-web-client",
    "tests/_fixtures/_git/wave-ui",
  ]);
  assert.deepEqual(coverage.presentInFixtures["jsx-vapor"], ["tests/_fixtures/_git/vue-jsx-vapor"]);
  for (const dialect of ["vue0-template", "vue1-template", "jsx-babel", "vue-quirks"]) {
    assert.deepEqual(coverage.presentInFixtures[dialect], []);
  }
});

test("dialect claims fail closed when evidence or unknown state drifts", () => {
  for (const mutate of [
    (value: typeof ledger) => {
      value.fixtures[0].dialectCoverage.state = "partial";
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/wave-ui"),
      ).dialectCoverage.evidence[0].selector = "stale";
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/vue2-elm"),
      ).dialectCoverage.evidence[0].file = "tests/missing.ts";
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/petite-vue"),
      ).dialectCoverage.evidence[0].dialects = ["made-up"];
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/vue-material"),
      ).dialectCoverage.evidence[0].revision = "0".repeat(40);
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/vue-material"),
      ).dialectCoverage.evidence[0].value = "^3.0.0";
    },
  ]) {
    const changed = structuredClone(ledger);
    mutate(changed);
    assert.throws(() => validateCompatibilityLedger(changed, context));
  }
});
