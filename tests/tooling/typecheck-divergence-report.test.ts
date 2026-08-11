import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  commitSha,
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
    updateJson(
      path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
      (preparation) => {
        const source = fs.readFileSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json"));
        preparation.baselineConfig = {
          path: ".generated/tsconfig.json",
          sizeBytes: source.byteLength,
          sha256: createHash("sha256").update(source).digest("hex"),
        };
      },
    );
    const preparationPath = path.join(fixture.reportDir, "fixture-typecheck-dependencies.json");
    const preparationRaw = fs.readFileSync(preparationPath);
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    assert.deepEqual(Object.keys(artifact).sort(), [
      "baseline",
      "budget",
      "divergence",
      "evidence",
      "preparation",
      "project",
      "revision",
      "schema",
      "seededMutation",
      "source",
      "tsconfig",
      "version",
    ]);
    assert.equal(artifact.schema, "vize.fixtureTypecheckDivergenceRun");
    assert.equal(artifact.version, 4);
    assert.equal(artifact.tsconfig, ".generated/tsconfig.json");
    assert.equal(artifact.evidence.commitSha, commitSha);
    assert.deepEqual(artifact.source, {
      payloadSha256: createHash("sha256").update(fs.readFileSync(fixture.outputPath)).digest("hex"),
      fileCount: 1,
      command: `${fixture.vize} check src/**/*.vue --format json --no-config --tsconfig tsconfig.json`,
      cwd: path.relative(root, fixture.fixtureRoot),
      durationMs: 1,
      peakRssBytes: 1,
      version: "vize 0.0.0",
    });
    assert.equal(artifact.preparation.schema, "vize.fixtureTypecheckDependencyInstall");
    assert.equal(artifact.preparation.version, 3);
    assert.equal(
      artifact.preparation.artifactSha256,
      createHash("sha256").update(preparationRaw).digest("hex"),
    );
    assert.deepEqual(artifact.preparation.baselineConfig, {
      path: ".generated/tsconfig.json",
      sizeBytes: fs.readFileSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json"))
        .byteLength,
      sha256: createHash("sha256")
        .update(fs.readFileSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json")))
        .digest("hex"),
    });
    assert.equal(artifact.seededMutation.tier, "sfc-script-ts2322");
    assert.equal(artifact.seededMutation.probeFile, ".vize-typecheck-parity-seed.vue");
    assert.deepEqual(
      artifact.seededMutation.states.map((state) => state.state),
      ["clean", "broken", "repaired"],
    );
    assert.equal(
      artifact.seededMutation.states[0].sourceSha256,
      artifact.seededMutation.states[2].sourceSha256,
    );
    assert.notEqual(
      artifact.seededMutation.states[0].sourceSha256,
      artifact.seededMutation.states[1].sourceSha256,
    );
    assert.equal(artifact.seededMutation.states[1].divergence.summary.sharedCount, 1);
    assert.equal(artifact.seededMutation.states[1].divergence.shared[0].code, 2322);
    assert.equal(
      artifact.seededMutation.states[1].divergence.shared[0].file,
      ".vize-typecheck-parity-seed.vue",
    );
    for (const state of artifact.seededMutation.states) {
      assert.equal(state.coverage.sharedVueFileCount, 1);
      assert.equal(state.coverage.verdict, "usable");
      assert.ok(state.vize.peakRssBytes > 0);
      assert.ok(state.baseline.peakRssBytes > 0);
    }
    assert.deepEqual(Object.keys(artifact.baseline).sort(), [
      "command",
      "configSha256",
      "configuration",
      "coverage",
      "durationMs",
      "exitCode",
      "peakRssBytes",
      "sourceConfigSha256",
      "stderrSha256",
      "stdoutSha256",
      "version",
    ]);
    assert.equal(artifact.baseline.exitCode, 2);
    assert.equal(artifact.baseline.version, "3.3.4");
    assert.ok(artifact.baseline.peakRssBytes > 0);
    assert.equal(
      artifact.baseline.sourceConfigSha256,
      createHash("sha256")
        .update(fs.readFileSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json")))
        .digest("hex"),
    );
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [],
      errorCount: 0,
      blockingErrorCount: 0,
      ignoredDeprecationErrorCount: 0,
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
      maxFalsePositiveRatio: 0,
      maxFalseNegativeRatio: 0,
      messageMismatchPassed: true,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "passed",
      passed: true,
    });
    assert.equal(artifact.divergence.summary.sharedCount, 1);
    assert.equal(artifact.divergence.summary.falsePositiveCount, 0);
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    const invocation = readJson(fixture.invocationPath);
    const baselineProject = path.join(fixture.reportDir, "fixture-vue-tsc", "tsconfig.json");
    assert.deepEqual(invocation, {
      cwd: fixture.fixtureRoot,
      args: ["--noEmit", "--pretty", "false", "--listFiles", "-p", baselineProject],
    });
    const fixtureBase = path
      .relative(path.dirname(baselineProject), fixture.fixtureRoot)
      .replaceAll("\\", "/");
    // The elk shape: the baseline config is generated into a dot-directory, which
    // a TypeScript wildcard segment never descends into, so it is globbed by name.
    const generatedBase = `${fixtureBase}/.generated`;
    assert.deepEqual(readJson(baselineProject), {
      extends: `${generatedBase}/tsconfig.json`,
      compilerOptions: { composite: false, incremental: false },
      files: [
        path
          .relative(path.dirname(baselineProject), path.join(fixture.fixtureRoot, "src/App.vue"))
          .replaceAll("\\", "/"),
      ],
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

test("typecheck divergence report fails closed on mismatched matrix artifacts", () => {
  const fixture = setup();
  try {
    const payloadPath = path.join(fixture.reportDir, "fixture-typechecker.json");
    updateJson(payloadPath, (payload) => (payload.project = "wrong-project"));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /artifact identity is invalid/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects evidence from another commit", () => {
  const fixture = setup();
  try {
    const result = run(fixture, { GITHUB_SHA: "c".repeat(40) });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /commit does not match GITHUB_SHA/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report requires exact dependency preparation evidence", () => {
  const cases: Array<
    [name: string, mutate: (fixture: ReturnType<typeof setup>) => void, message: RegExp]
  > = [
    [
      "missing artifact",
      (fixture) => fs.rmSync(path.join(fixture.reportDir, "fixture-typecheck-dependencies.json")),
      /preparation evidence is missing/,
    ],
    [
      "stale commit",
      (fixture) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => (artifact.evidence.commitSha = "c".repeat(40)),
        ),
      /commit or runtime is stale/,
    ],
    [
      "stale revision",
      (fixture) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => (artifact.revision = "d".repeat(40)),
        ),
      /preparation identity is invalid/,
    ],
    [
      "unknown field",
      (fixture) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => (artifact.trustMe = true),
        ),
      /preparation evidence shape is invalid/,
    ],
    [
      "changed lockfile",
      (fixture) => fs.appendFileSync(path.join(fixture.fixtureRoot, "pnpm-lock.yaml"), "# stale\n"),
      /preparation lockfile is stale/,
    ],
    [
      "changed baseline config",
      (fixture) => fs.appendFileSync(path.join(fixture.fixtureRoot, "tsconfig.json"), " "),
      /preparation baseline config is stale/,
    ],
    [
      "unreviewed install command",
      (fixture) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => artifact.install.command.push("--force"),
        ),
      /preparation install evidence is invalid/,
    ],
    [
      "unexpected prepare command",
      (fixture) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => {
            artifact.baselinePrepare = {
              command: ["pnpm", "prepare"],
              durationMs: 1,
              exitCode: 0,
              stdoutSha256: "0".repeat(64),
              stderrSha256: "0".repeat(64),
            };
          },
        ),
      /preparation command is unexpected/,
    ],
  ];
  for (const [name, mutate, message] of cases) {
    const fixture = setup();
    try {
      mutate(fixture);
      const result = run(fixture);
      assert.equal(result.status, 1, name);
      assert.match(result.stderr, message, name);
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
