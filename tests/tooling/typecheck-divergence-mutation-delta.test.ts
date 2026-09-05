import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  run,
  setup,
  writeJson,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

test("typecheck divergence mutation delta counts documented one-sided diagnostics by side", () => {
  const fixture = setup({ baselineMutation: "missing" });
  try {
    const ledgerPath = path.join(fixture.reportDir, "documented-differences.json");
    writeJson(ledgerPath, {
      schema: "vize.compatDocumentedDifferences",
      version: 1,
      differences: [
        {
          project: "fixture",
          file: "src/App.vue",
          severity: "error",
          line: 3,
          column: 1,
          vize: {
            code: 2322,
            message: "Type 'number' is not assignable to type 'string'.",
          },
          baseline: null,
          issue: 5722,
          reason: "The seeded mutation fixture intentionally covers Vize-only delta accounting.",
        },
      ],
    });

    const result = run(fixture, {}, [
      "--budget-mode",
      "record-only",
      "--documented-differences",
      ledgerPath,
    ]);
    assert.equal(result.status, 0, result.stderr);

    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    const brokenState = artifact.mutationOracle.states[1];
    assert.equal(brokenState.documentedDifferenceCount, 1);
    assert.equal(brokenState.vizeDiagnosticCount, 1);
    assert.equal(brokenState.baselineDiagnosticCount, 0);
    assert.equal(brokenState.falsePositiveCount, 0);
    assert.equal(brokenState.falseNegativeCount, 0);
  } finally {
    cleanup(fixture);
  }
});
