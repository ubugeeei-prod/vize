import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../tools/support/compat/fixtures/fixture-compatibility-ledger.ts";
import { createCompatibilityReport } from "../../tools/support/compat/fixtures/fixture-compatibility-report.ts";

const context = createCompatibilityContext();
const ledger = readCompatibilityLedger();

// Presence floors: a fixture may gain claims without editing this test, but
// a recorded claim never disappears silently.
const presenceFloor: Record<string, readonly string[]> = {
  "vue2-sfc": [
    "bootstrap-vue",
    "cube-ui",
    "element",
    "mobile-web-best-practice",
    "vue-element-admin",
    "vue-select",
    "vue2-elm",
    "vux",
  ],
  "vue2.7-sfc": ["vue-material"],
  "petite-vue": ["petite-vue", "wakapi"],
  "pug-template": ["dho-web-client", "wave-ui"],
  "jsx-vapor": ["vue-jsx-vapor"],
  "jsx-babel": ["ant-design-vue", "element-plus", "vuetify"],
};

test("pinned project dialect presence stays distinct from per-file coverage", () => {
  validateCompatibilityLedger(ledger, context);
  const coverage = createCompatibilityReport(ledger, context).dialectCoverage;
  assert.equal(coverage.unknownFixtureCount + coverage.partialFixtureCount, ledger.fixtures.length);
  // Package scans record most registered projects; the rest stay unknown.
  assert.ok(coverage.partialFixtureCount > coverage.unknownFixtureCount);
  for (const [dialect, fixtures] of Object.entries(presenceFloor)) {
    for (const fixture of fixtures) {
      assert.ok(
        coverage.presentInFixtures[dialect].includes(`tests/_fixtures/_git/${fixture}`),
        `${dialect}: ${fixture}`,
      );
    }
  }
  // No pinned real project uses these; the synthetic dialect corpus covers them.
  for (const dialect of ["vue0-template", "vue1-template", "vue-quirks"]) {
    assert.deepEqual(coverage.presentInFixtures[dialect], []);
  }
});

test("dialect claims fail closed when evidence or unknown state drifts", () => {
  for (const mutate of [
    (value: typeof ledger) => {
      value.fixtures.find((row) => row.dialectCoverage.state === "unknown").dialectCoverage.state =
        "partial";
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) => row.dialectCoverage.state === "partial").dialectCoverage.state =
        "unknown";
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
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/vue-material"),
      ).dialectCoverage.evidence[0].field = "devDependencies.lodash";
    },
    (value: typeof ledger) => {
      value.fixtures.find((row) =>
        row.fixturePath.endsWith("/vue-material"),
      ).dialectCoverage.evidence[0].file = "node_modules/vue/package.json";
    },
  ]) {
    const changed = structuredClone(ledger);
    mutate(changed);
    assert.throws(() => validateCompatibilityLedger(changed, context));
  }
});
