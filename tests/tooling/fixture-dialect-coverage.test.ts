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
  assert.equal(coverage.unknownFixtureCount, 137);
  assert.equal(coverage.partialFixtureCount, 9);
  assert.deepEqual(coverage.presentInFixtures["vue2-sfc"], [
    "tests/_fixtures/_git/mobile-web-best-practice",
    "tests/_fixtures/_git/vue-element-admin",
    "tests/_fixtures/_git/vue2-elm",
  ]);
  assert.deepEqual(coverage.presentInFixtures["petite-vue"], [
    "tests/_fixtures/_git/petite-vue",
    "tests/_fixtures/_git/wakapi",
  ]);
  assert.deepEqual(coverage.presentInFixtures["pug-template"], [
    "tests/_fixtures/_git/dho-web-client",
    "tests/_fixtures/_git/wave-ui",
  ]);
  assert.deepEqual(coverage.presentInFixtures["jsx-vapor"], ["tests/_fixtures/_git/vue-jsx-vapor"]);
  for (const dialect of ["vue0-sfc", "vue1-sfc", "vue2.7-sfc", "jsx-babel", "vue-quirks"]) {
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
  ]) {
    const changed = structuredClone(ledger);
    mutate(changed);
    assert.throws(() => validateCompatibilityLedger(changed, context));
  }
});
