import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  commitSha,
  pnpmInstallCommand,
  readJson,
  root,
  run,
  setup,
  updateJson,
  writeJson,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

test("typecheck divergence report binds baseline evidence to the matrix artifact", () => {
  const fixture = setup();
  try {
    fs.mkdirSync(path.join(fixture.fixtureRoot, ".generated"));
    fs.writeFileSync(
      path.join(fixture.fixtureRoot, ".generated/tsconfig.json"),
      '{"compilerOptions":{"strict":true}}\n',
    );
    updateJson(
      fixture.registryPath,
      (registry) =>
        (registry.projects[0].typecheckPerformance.baseline = {
          tsconfig: ".generated/tsconfig.json",
        }),
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    assert.deepEqual(Object.keys(artifact).sort(), [
      "baseline",
      "budget",
      "divergence",
      "enforcement",
      "evidence",
      "mutationOracle",
      "preparation",
      "project",
      "revision",
      "schema",
      "source",
      "tsconfig",
      "version",
    ]);
    assert.equal(artifact.schema, "vize.fixtureTypecheckDivergenceRun");
    assert.equal(artifact.version, 7);
    assert.equal(artifact.tsconfig, ".generated/tsconfig.json");
    assert.equal(artifact.evidence.commitSha, commitSha);
    assert.deepEqual(artifact.enforcement, { budgetMode: "enforce" });
    const dependencyPath = path.join(fixture.reportDir, "fixture-typecheck-dependencies.json");
    assert.deepEqual(artifact.preparation, {
      schema: "vize.fixtureTypecheckPreparationEvidence",
      version: 1,
      payloadSha256: createHash("sha256").update(fs.readFileSync(dependencyPath)).digest("hex"),
      packageManager: { name: "pnpm", version: "10.0.0" },
      lockfile: {
        path: "pnpm-lock.yaml",
        sizeBytes: fs.readFileSync(path.join(fixture.fixtureRoot, "pnpm-lock.yaml")).byteLength,
        sha256: createHash("sha256")
          .update(fs.readFileSync(path.join(fixture.fixtureRoot, "pnpm-lock.yaml")))
          .digest("hex"),
      },
      install: {
        command: pnpmInstallCommand,
        exitCode: 0,
        stdoutSha256: createHash("sha256").update("installed").digest("hex"),
        stderrSha256: createHash("sha256").update("").digest("hex"),
      },
      baselinePrepare: null,
    });
    assert.deepEqual(artifact.source, {
      payloadSha256: createHash("sha256").update(fs.readFileSync(fixture.outputPath)).digest("hex"),
      fileCount: 1,
    });
    assert.deepEqual(Object.keys(artifact.baseline).sort(), [
      "ambient",
      "command",
      "configSha256",
      "configuration",
      "coverage",
      "coverageCommand",
      "coverageDurationMs",
      "coverageExitCode",
      "coverageRunError",
      "coverageStderrSha256",
      "coverageStdoutSha256",
      "durationMs",
      "exitCode",
      "runError",
      "sourceConfigSha256",
      "stderrSha256",
      "stdoutSha256",
      "version",
    ]);
    assert.equal(artifact.baseline.exitCode, 2);
    assert.equal(artifact.baseline.coverageExitCode, 0);
    assert.equal(artifact.baseline.runError, null);
    assert.equal(artifact.baseline.coverageRunError, null);
    assert.equal(artifact.baseline.version, "3.3.4");
    assert.equal(
      artifact.baseline.sourceConfigSha256,
      createHash("sha256")
        .update(fs.readFileSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json")))
        .digest("hex"),
    );
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [],
      errorCount: 0,
      unusableReason: null,
      verdict: "usable",
    });
    assert.deepEqual(artifact.baseline.coverage, {
      baselineVueFileCount: 1,
      baselineVueFilesSha256: createHash("sha256").update("src/App.vue\n").digest("hex"),
      ignoredDependencyVueFileCount: 0,
      ignoredDependencyVueFilesSha256: createHash("sha256").update("").digest("hex"),
      ignoredSupportVueFileCount: 0,
      ignoredSupportVueFilesSha256: createHash("sha256").update("").digest("hex"),
      missingVueFiles: [],
      sharedVueFileCount: 1,
      unexpectedVueFiles: [],
      unusableReason: null,
      verdict: "usable",
      vizeVueFileCount: 1,
      vizeVueFilesSha256: createHash("sha256").update("src/App.vue\n").digest("hex"),
    });
    assert.match(artifact.baseline.stdoutSha256, /^[0-9a-f]{64}$/);
    assert.match(artifact.baseline.stderrSha256, /^[0-9a-f]{64}$/);
    assert.match(artifact.baseline.coverageStdoutSha256, /^[0-9a-f]{64}$/);
    assert.match(artifact.baseline.coverageStderrSha256, /^[0-9a-f]{64}$/);
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "passed",
      passed: true,
    });
    assert.equal(artifact.divergence.summary.sharedCount, 1);
    assert.equal(artifact.divergence.summary.falsePositiveCount, 0);
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    assert.equal(artifact.mutationOracle.schema, "vize.fixtureTypecheckSeededMutationOracle");
    assert.equal(artifact.mutationOracle.version, 1);
    assert.equal(artifact.mutationOracle.verdict, "passed");
    assert.equal(artifact.mutationOracle.passed, true);
    assert.equal(artifact.mutationOracle.file, "src/App.vue");
    assert.deepEqual(artifact.mutationOracle.span, { line: 3, column: 1 });
    assert.deepEqual(
      artifact.mutationOracle.states.map((state) => state.name),
      ["clean", "broken", "repaired"],
    );
    assert.equal(artifact.mutationOracle.states[1].sharedCount, 1);
    assert.equal(artifact.mutationOracle.states[1].falsePositiveCount, 0);
    assert.equal(artifact.mutationOracle.states[1].falseNegativeCount, 0);
    assert.equal(
      artifact.mutationOracle.states[2].sourceSha256,
      artifact.mutationOracle.states[0].sourceSha256,
    );
    const invocation = readJson(fixture.invocationPath);
    const baselineProject = path.join(
      fixture.fixtureRoot,
      ".generated",
      ".vize-baseline",
      "fixture-vue-tsc.tsconfig.json",
    );
    const baselineArtifact = path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json");
    assert.deepEqual(invocation, {
      cwd: fixture.fixtureRoot,
      args: ["--noEmit", "--pretty", "false", "-p", baselineProject],
    });
    assert.match(artifact.baseline.coverageCommand, /--listFilesOnly/);
    assert.equal(
      fs.readFileSync(baselineArtifact, "utf8"),
      fs.readFileSync(baselineProject, "utf8"),
    );
    // The elk shape: the baseline config is generated into a dot-directory, which
    // a TypeScript wildcard segment never descends into, so it is globbed by name.
    // The src root carries non-Vue support files without expanding the Vue corpus.
    const generatedBase = "..";
    const sourceBase = "../../src";
    const fixtureBase = "../..";
    assert.deepEqual(readJson(baselineArtifact), {
      extends: "../tsconfig.json",
      compilerOptions: {
        ignoreDeprecations: "6.0",
        rootDir: fixtureBase,
      },
      files: [`${fixtureBase}/src/App.vue`],
      include: [
        `${generatedBase}/**/*.d.ts`,
        `${sourceBase}/**/*.d.ts`,
        `${sourceBase}/**/*.ts`,
        `${sourceBase}/**/*.tsx`,
        `${sourceBase}/**/*.mts`,
        `${sourceBase}/**/*.cts`,
        `${sourceBase}/**/*.js`,
        `${sourceBase}/**/*.jsx`,
        `${sourceBase}/**/*.mjs`,
        `${sourceBase}/**/*.cjs`,
        `${sourceBase}/**/*.json`,
      ],
      exclude: [
        `${generatedBase}/**/node_modules/**`,
        `${generatedBase}/**/dist/**`,
        `${sourceBase}/**/node_modules/**`,
        `${sourceBase}/**/dist/**`,
      ],
      references: [],
    });
    const markdown = fs.readFileSync(
      path.join(fixture.reportDir, "fixture-typecheck-divergence.md"),
      "utf8",
    );
    assert.match(markdown, /vue-tsc excluded project-level: 0/);
    assert.match(markdown, /Seeded mutation oracle: passed \(src\/App\.vue:3:1\)/);
    assert.match(markdown, new RegExp(`Digest: ${artifact.divergence.sha256}`));
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence progress logging is opt-in", () => {
  const fixture = setup({ vizeDiagnostics: [], baselineOutput: "" });
  try {
    const quiet = run(fixture);
    assert.equal(quiet.status, 0, quiet.stderr);
    assert.doesNotMatch(quiet.stderr, /^\[typecheck-divergence\]/mu);

    const result = run(fixture, { VIZE_TYPECHECK_DIVERGENCE_PROGRESS: "1" });
    assert.equal(result.status, 0, result.stderr);
    const progressLines = result.stderr.trimEnd().split("\n");
    assert.equal(progressLines.length, 5);
    assert.equal(progressLines[0], "[typecheck-divergence] start projectId=fixture timeoutMs=5000");
    assert.equal(progressLines[1], "[typecheck-divergence] run projectId=fixture command=baseline");
    assert.match(
      progressLines[2],
      /^\[typecheck-divergence\] finish projectId=fixture command=baseline durationMs=\d+ status=2$/u,
    );
    assert.equal(progressLines[3], "[typecheck-divergence] run projectId=fixture command=coverage");
    assert.match(
      progressLines[4],
      /^\[typecheck-divergence\] finish projectId=fixture command=coverage durationMs=\d+ status=0$/u,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report skips shards without typecheck performance targets", () => {
  const fixture = setup();
  try {
    updateJson(
      fixture.registryPath,
      (registry) => (registry.projects[0].typecheckPerformance.enabled = false),
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /No typecheck performance projects selected/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report falls back to Vize files when corpus globs are absent", () => {
  const fixture = setup();
  try {
    updateJson(fixture.registryPath, (registry) => {
      delete registry.projects[0].vueGlobs;
      delete registry.projects[0].typecheckPerformance.corpusGlobs;
    });
    const result = run(fixture, {}, ["--budget-mode", "record-only"]);
    assert.equal(result.status, 0, result.stderr);

    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    assert.equal(artifact.baseline.coverage.verdict, "usable");
    assert.equal(artifact.baseline.coverage.vizeVueFileCount, 1);
    assert.equal(artifact.baseline.coverage.baselineVueFileCount, 1);
    assert.equal(
      artifact.mutationOracle.unusableReason,
      "seeded mutation requires configured typecheck corpus globs to rerun Vize",
    );
    const config = readJson(path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json"));
    assert.equal(config.include.includes("../src/**/*.ts"), true);
  } finally {
    cleanup(fixture);
  }
});

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
