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
  updateJson,
  writeJson,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

test("typecheck divergence report rejects parsed output that differs from stdout", () => {
  const fixture = setup();
  try {
    const payloadPath = path.join(fixture.reportDir, "fixture-typechecker.json");
    updateJson(payloadPath, (payload) => (payload.parsed.errorCount = 2));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /stdout does not match parsed output/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects a mismatched matrix file count", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => (summary.projects[0].runs[0].fileCount = 2));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /file count is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects mutated raw typechecker coverage", () => {
  const fixture = setup();
  try {
    updateJson(fixture.outputPath, (payload) => {
      payload.typecheckerCoverage.checked.sha256 = "0".repeat(64);
    });
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /typechecker coverage is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects mutated summary coverage", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => {
      summary.projects[0].runs[0].coverage.requestedFileCount = 0;
    });
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /summary coverage is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects a mismatched matrix status", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => (summary.projects[0].runs[0].status = "ok"));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /status is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report binds exact Vize execution evidence", () => {
  for (const [name, mutate] of [
    ["command", (run) => (run.command = "another-vize check src/**/*.vue")],
    ["cwd", (run) => (run.cwd = "tests/_fixtures/another-project")],
    ["duration", (run) => (run.durationMs = -1)],
    ["peak RSS", (run) => (run.peakRssBytes = 0)],
  ] as const) {
    const fixture = setup();
    try {
      updateJson(path.join(fixture.reportDir, "summary.json"), (summary) => {
        mutate(summary.projects[0].runs[0]);
      });
      const result = run(fixture);
      assert.equal(result.status, 1, name);
      assert.match(result.stderr, /execution evidence is invalid/, name);
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
        false,
        name,
      );
    } finally {
      cleanup(fixture);
    }
  }
});

test("typecheck divergence report rejects a seeded diagnostic code mutation", () => {
  const fixture = setup();
  try {
    const source = fs.readFileSync(fixture.vize, "utf8");
    fs.writeFileSync(fixture.vize, source.replaceAll("TS2322", "TS2323"));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Seeded broken diagnostic did not match exactly/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );
    assert.equal(
      fs.existsSync(path.join(fixture.fixtureRoot, ".vize-typecheck-parity-seed.vue")),
      false,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects an artifact outside the reported directory", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(
      summaryPath,
      (summary) =>
        (summary.projects[0].runs[0].outputPath = path.relative(
          root,
          path.join(fixture.reportDir, "nested", "fixture-typechecker.json"),
        )),
    );
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /output path is invalid/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects invalid performance budgets", () => {
  for (const [field, value, message] of [
    ["hangTimeoutMs", 0, /hangTimeoutMs must be a positive safe integer/],
    ["maxFalsePositiveRatio", Number.NaN, /maxFalsePositiveRatio must be a finite number/],
    ["maxFalseNegativeRatio", 1.1, /maxFalseNegativeRatio must be a finite number/],
    ["maxFalsePositiveRatio", 0.01, /FP\/FN ratios must both be 0/],
  ] as const) {
    const fixture = setup();
    try {
      const registry = readJson(fixture.registryPath);
      registry.projects[0].typecheckPerformance[field] = value;
      writeJson(fixture.registryPath, registry);
      const result = run(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
    } finally {
      cleanup(fixture);
    }
  }
});

test("typecheck divergence report rejects unsupported baseline exits and output", () => {
  for (const [body, message] of [
    ["process.exit(1);", /unsupported status 1/],
    ["process.stderr.write('prefix error TS1: bad\\n'); process.exit(2);", /unparseable/],
  ] as const) {
    const fixture = setup();
    try {
      writeVueTsc(fixture.vueTsc, body);
      const result = run(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
    } finally {
      cleanup(fixture);
    }
  }
});

test("typecheck divergence report evaluates every selected project in one shard", () => {
  const fixture = setup();
  try {
    const registry = readJson(fixture.registryPath);
    const secondProject = { ...registry.projects[0], id: "fixture-two" };
    registry.projects.push(secondProject);
    writeJson(fixture.registryPath, registry);

    const secondOutput = path.join(fixture.reportDir, "fixture-two-typechecker.json");
    const output = readJson(fixture.outputPath);
    output.project = secondProject.id;
    writeJson(secondOutput, output);

    const preparation = readJson(
      path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
    );
    preparation.project = secondProject.id;
    writeJson(path.join(fixture.reportDir, "fixture-two-typecheck-dependencies.json"), preparation);

    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => {
      const secondSummary = structuredClone(summary.projects[0]);
      secondSummary.id = secondProject.id;
      secondSummary.runs[0].outputPath = path.relative(root, secondOutput);
      summary.projects.push(secondSummary);
    });

    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    for (const project of ["fixture", "fixture-two"]) {
      const artifactPath = path.join(fixture.reportDir, `${project}-typecheck-divergence.json`);
      assert.equal(readJson(artifactPath).project, project);
    }
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report requires at least one performance project per shard", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(fixture.registryPath, '{"projects":[]}\n');
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Expected at least one.*found 0/);
  } finally {
    cleanup(fixture);
  }
});
