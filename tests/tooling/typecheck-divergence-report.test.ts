import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  commitSha,
  readJson,
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
    assert.equal(artifact.version, 4);
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
        command: ["pnpm", "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
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
      "command",
      "configSha256",
      "configuration",
      "coverage",
      "durationMs",
      "exitCode",
      "sourceConfigSha256",
      "stderrSha256",
      "stdoutSha256",
      "version",
    ]);
    assert.equal(artifact.baseline.exitCode, 2);
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
      args: ["--noEmit", "--pretty", "false", "--listFiles", "-p", baselineProject],
    });
    assert.equal(
      fs.readFileSync(baselineArtifact, "utf8"),
      fs.readFileSync(baselineProject, "utf8"),
    );
    // The elk shape: the baseline config is generated into a dot-directory, which
    // a TypeScript wildcard segment never descends into, so it is globbed by name.
    const fixtureBase = "../..";
    const generatedBase = "..";
    assert.deepEqual(readJson(baselineArtifact), {
      extends: "../tsconfig.json",
      compilerOptions: {
        ignoreDeprecations: "6.0",
        rootDir: fixtureBase,
      },
      files: [`${fixtureBase}/src/App.vue`],
      // #3738: ambient declarations are the fixture's type environment, and a
      // `files`-only program drops every one of them.
      include: [`${fixtureBase}/**/*.d.ts`, `${generatedBase}/**/*.d.ts`],
      exclude: [
        `${fixtureBase}/**/node_modules/**`,
        `${fixtureBase}/**/dist/**`,
        `${generatedBase}/**/node_modules/**`,
        `${generatedBase}/**/dist/**`,
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
