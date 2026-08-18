import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { cleanup, readJson, run, setup } from "./_helpers/typecheck-divergence-report-fixture.ts";

function artifactPath(fixture: ReturnType<typeof setup>, extension: string) {
  return path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);
}

test("seeded mutation oracle accepts a shared probe with shifted compiler coordinates", () => {
  const fixture = setup({
    baselineOutput: "",
    baselineFiles: ["src/App.vue"],
    baselineMutation: "shifted",
    vizeDiagnostics: [],
    vizeMutation: "shifted",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const oracle = readJson(artifactPath(fixture, "json")).mutationOracle;
    assert.equal(oracle.passed, true);
    assert.equal(oracle.expectedDiagnosticMatched, true);
    assert.equal(oracle.file, "src/App.vue");
    assert.deepEqual(oracle.span, { line: 3, column: 1 });
    assert.equal(oracle.states.length, 3);
    const [cleanState, brokenState, repairedState] = oracle.states;
    assert.equal(cleanState.sharedCount, 0);
    assert.equal(cleanState.messageMismatchCount, 0);
    assert.equal(brokenState.sharedCount, 1);
    assert.equal(brokenState.messageMismatchCount, 0);
    assert.equal(repairedState.sharedCount, 0);
    assert.equal(repairedState.messageMismatchCount, 0);
    assert.notEqual(brokenState.sourceSha256, cleanState.sourceSha256);
    assert.equal(repairedState.sourceSha256, cleanState.sourceSha256);
  } finally {
    cleanup(fixture);
  }
});
