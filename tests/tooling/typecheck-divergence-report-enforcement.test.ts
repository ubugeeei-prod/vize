import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  root,
  run,
  setup,
  writeJson,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

test("typecheck divergence report writes all selected artifacts before enforcing budgets", () => {
  const fixture = setup({
    vizeDiagnostics: ["error:1:1 [TS2322] shared", "error:1:2 [TS2322] extra"],
  });
  try {
    const registry = readJson(fixture.registryPath);
    registry.projects.push({ ...registry.projects[0], id: "second" });
    writeJson(fixture.registryPath, registry);

    const summaryPath = path.join(fixture.reportDir, "summary.json");
    const summary = readJson(summaryPath);
    const secondOutput = path.join(fixture.reportDir, "second-typechecker.json");
    summary.projects.push({
      ...summary.projects[0],
      id: "second",
      runs: summary.projects[0].runs.map((run: any) => ({
        ...run,
        outputPath: path.relative(root, secondOutput),
      })),
    });
    writeJson(summaryPath, summary);

    const payload = readJson(fixture.outputPath);
    payload.project = "second";
    writeJson(secondOutput, payload);
    const preparation = readJson(
      path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
    );
    preparation.project = "second";
    writeJson(path.join(fixture.reportDir, "second-typecheck-dependencies.json"), preparation);

    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Typecheck divergence budget breached for fixture/);
    assert.match(result.stderr, /Typecheck divergence budget breached for second/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      true,
    );
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "second-typecheck-divergence.json")),
      true,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report requires a false-negative budget", () => {
  const fixture = setup();
  try {
    const registry = readJson(fixture.registryPath);
    delete registry.projects[0].typecheckPerformance.maxFalseNegativeRatio;
    writeJson(fixture.registryPath, registry);
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /maxFalseNegativeRatio must be a finite number/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report accepts vue-tsc diagnostic status 1", () => {
  const fixture = setup({ baselineExitCode: 1 });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    assert.equal(artifact.baseline.exitCode, 1);
    assert.equal(artifact.budget.verdict, "passed");
  } finally {
    cleanup(fixture);
  }
});
