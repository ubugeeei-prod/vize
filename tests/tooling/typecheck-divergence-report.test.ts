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
  writeVueTsc,
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
      "evidence",
      "project",
      "revision",
      "schema",
      "source",
      "tsconfig",
      "version",
    ]);
    assert.equal(artifact.schema, "vize.fixtureTypecheckDivergenceRun");
    assert.equal(artifact.version, 3);
    assert.equal(artifact.tsconfig, ".generated/tsconfig.json");
    assert.equal(artifact.evidence.commitSha, commitSha);
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
    const invocation = readJson(fixture.invocationPath);
    const baselineProject = path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json");
    assert.deepEqual(invocation, {
      cwd: fixture.fixtureRoot,
      args: ["--noEmit", "--pretty", "false", "--listFiles", "-p", baselineProject],
    });
    const fixtureBase = path.relative(fixture.reportDir, fixture.fixtureRoot).replaceAll("\\", "/");
    // The elk shape: the baseline config is generated into a dot-directory, which
    // a TypeScript wildcard segment never descends into, so it is globbed by name.
    const generatedBase = `${fixtureBase}/.generated`;
    assert.deepEqual(readJson(baselineProject), {
      extends: `${generatedBase}/tsconfig.json`,
      files: [
        path
          .relative(fixture.reportDir, path.join(fixture.fixtureRoot, "src/App.vue"))
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

test("typecheck divergence report requires one performance project per shard", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(fixture.registryPath, '{"projects":[]}\n');
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Expected exactly one.*found 0/);
  } finally {
    cleanup(fixture);
  }
});
