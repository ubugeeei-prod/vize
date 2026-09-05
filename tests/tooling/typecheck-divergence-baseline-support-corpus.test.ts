import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  run,
  setup,
  sharedBaselineOutput,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

const artifactPath = (fixture: ReturnType<typeof setup>, extension: string) =>
  path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);

test("transitive dot-directory Vue support does not expand the authored fixture corpus", () => {
  const fixture = setup({
    baselineFiles: ["src/App.vue", "docs/.vitepress/components/Support.vue"],
    baselineOutput:
      `${sharedBaselineOutput}` +
      "docs/.vitepress/components/Support.vue(1,1): error TS2322: support only\n",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    assert.equal(artifact.divergence.summary.baselineExcludedSupportVueCount, 1);
    const coverage = artifact.baseline.coverage;
    assert.equal(coverage.verdict, "usable");
    assert.equal(coverage.baselineVueFileCount, 1);
    assert.equal(coverage.ignoredSupportVueFileCount, 1);
    assert.deepEqual(coverage.missingVueFiles, []);
    assert.deepEqual(coverage.unexpectedVueFiles, []);
    const markdown = fs.readFileSync(artifactPath(fixture, "md"), "utf8");
    assert.match(markdown, /^vue-tsc excluded support Vue: 1$/m);
    assert.match(markdown, /^Ignored support Vue files: 1$/m);
  } finally {
    cleanup(fixture);
  }
});

test("transitive Vue support outside corpus roots does not expand the authored fixture corpus", () => {
  const fixture = setup({
    baselineFiles: ["src/App.vue", "docs/components/Support.vue"],
    baselineOutput:
      `${sharedBaselineOutput}` + "docs/components/Support.vue(1,1): error TS2322: support only\n",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    assert.equal(artifact.divergence.summary.baselineExcludedSupportVueCount, 1);
    const coverage = artifact.baseline.coverage;
    assert.equal(coverage.verdict, "usable");
    assert.equal(coverage.baselineVueFileCount, 1);
    assert.equal(coverage.ignoredSupportVueFileCount, 1);
    assert.deepEqual(coverage.missingVueFiles, []);
    assert.deepEqual(coverage.unexpectedVueFiles, []);
  } finally {
    cleanup(fixture);
  }
});
