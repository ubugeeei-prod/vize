import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const script = path.join(root, "tools", "fixtures", "typecheck-divergence-report.mjs");
const commitSha = "a".repeat(40);

function setup() {
  const fixtureRoot = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "typecheck-divergence-"),
  );
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-divergence-report-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-divergence-vue-tsc-"));
  const fixturePath = path.relative(root, fixtureRoot);
  const project = {
    id: "fixture",
    fixturePath,
    revision: "b".repeat(40),
    vueGlobs: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    typecheckPerformance: {
      enabled: true,
      compareTo: "vue-tsc",
      hangTimeoutMs: 5_000,
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
    },
  };
  fs.mkdirSync(path.join(fixtureRoot, "src"));
  fs.writeFileSync(path.join(fixtureRoot, "tsconfig.json"), "{}\n");
  fs.writeFileSync(path.join(fixtureRoot, "src", "App.vue"), "<template />\n");
  const registryPath = path.join(fixtureRoot, "registry.json");
  fs.writeFileSync(registryPath, `${JSON.stringify({ projects: [project] }, null, 2)}\n`);
  const outputPath = path.join(reportDir, "fixture-typechecker.json");
  const parsed = {
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
    files: [{ file: "src/App.vue", diagnostics: ["error:1:1 [TS2322] shared"] }],
  };
  fs.writeFileSync(
    outputPath,
    `${JSON.stringify(
      {
        schema: "vize.fixtureToolRun",
        version: 1,
        project: "fixture",
        tool: "typechecker",
        exitCode: 1,
        stdout: JSON.stringify(parsed),
        stderr: "",
        parsed,
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    path.join(reportDir, "summary.json"),
    `${JSON.stringify(
      {
        schema: "vize.fixtureToolMatrixReport",
        version: 2,
        evidence: {
          commitSha,
          runtime: { name: "node", version: process.versions.node },
          machine: {
            platform: process.platform,
            arch: process.arch,
            cpuModel: "synthetic",
            logicalCpuCount: 1,
            totalMemoryBytes: 1,
          },
        },
        projects: [
          {
            id: "fixture",
            revision: project.revision,
            runs: [
              {
                tool: "typechecker",
                status: "findings",
                exitCode: 1,
                fileCount: 1,
                outputPath: "nested/fixture-typechecker.json",
              },
            ],
          },
        ],
      },
      null,
      2,
    )}\n`,
  );
  const vueTsc = path.join(fakeDir, "vue-tsc.mjs");
  writeVueTsc(
    vueTsc,
    'process.stdout.write("src/App.vue(1,1): error TS2322: shared\\n"); process.exit(2);',
  );
  return { fixtureRoot, reportDir, fakeDir, registryPath, vueTsc, project };
}

function writeVueTsc(pathname: string, runBody: string) {
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node\nif (process.argv.includes("--version")) { console.log("3.3.4"); process.exit(0); }\n${runBody}\n`,
  );
  fs.chmodSync(pathname, 0o755);
}

function run(fixture: ReturnType<typeof setup>, env: NodeJS.ProcessEnv = {}) {
  return spawnSync(
    process.execPath,
    [
      script,
      "--registry",
      fixture.registryPath,
      "--report-dir",
      fixture.reportDir,
      "--vue-tsc-bin",
      fixture.vueTsc,
    ],
    { cwd: root, encoding: "utf8", env: { ...process.env, GITHUB_SHA: commitSha, ...env } },
  );
}

function cleanup(fixture: ReturnType<typeof setup>) {
  fs.rmSync(fixture.fixtureRoot, { recursive: true, force: true });
  fs.rmSync(fixture.reportDir, { recursive: true, force: true });
  fs.rmSync(fixture.fakeDir, { recursive: true, force: true });
}

test("typecheck divergence report binds baseline evidence to the matrix artifact", () => {
  const fixture = setup();
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = JSON.parse(
      fs.readFileSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"), "utf8"),
    );
    assert.deepEqual(Object.keys(artifact), [
      "schema",
      "version",
      "project",
      "revision",
      "tsconfig",
      "evidence",
      "baseline",
      "budget",
      "divergence",
    ]);
    assert.equal(artifact.schema, "vize.fixtureTypecheckDivergenceRun");
    assert.equal(artifact.evidence.commitSha, commitSha);
    assert.equal(artifact.baseline.exitCode, 2);
    assert.equal(artifact.baseline.version, "3.3.4");
    assert.ok(artifact.baseline.durationMs >= 0);
    assert.match(artifact.baseline.stdoutSha256, /^[0-9a-f]{64}$/);
    assert.match(artifact.baseline.stderrSha256, /^[0-9a-f]{64}$/);
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      passed: true,
    });
    assert.equal(artifact.divergence.summary.sharedCount, 1);
    assert.equal(artifact.divergence.summary.falsePositiveCount, 0);
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    const markdown = fs.readFileSync(
      path.join(fixture.reportDir, "fixture-typecheck-divergence.md"),
      "utf8",
    );
    assert.match(markdown, /Budget passed: true/);
    assert.match(markdown, new RegExp(`Digest: ${artifact.divergence.sha256}`));
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report records an unconfigured false-negative budget", () => {
  const fixture = setup();
  try {
    const registry = JSON.parse(fs.readFileSync(fixture.registryPath, "utf8"));
    delete registry.projects[0].typecheckPerformance.maxFalseNegativeRatio;
    fs.writeFileSync(fixture.registryPath, `${JSON.stringify(registry, null, 2)}\n`);
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = JSON.parse(
      fs.readFileSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"), "utf8"),
    );
    assert.equal(artifact.budget.maxFalseNegativeRatio, null);
    assert.equal(artifact.budget.falseNegativePassed, null);
    assert.equal(artifact.budget.passed, null);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report fails closed on mismatched matrix artifacts", () => {
  const fixture = setup();
  try {
    const payloadPath = path.join(fixture.reportDir, "fixture-typechecker.json");
    const payload = JSON.parse(fs.readFileSync(payloadPath, "utf8"));
    payload.project = "wrong-project";
    fs.writeFileSync(payloadPath, `${JSON.stringify(payload, null, 2)}\n`);
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
    const payload = JSON.parse(fs.readFileSync(payloadPath, "utf8"));
    payload.parsed.errorCount = 2;
    fs.writeFileSync(payloadPath, `${JSON.stringify(payload, null, 2)}\n`);
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
    const summary = JSON.parse(fs.readFileSync(summaryPath, "utf8"));
    summary.projects[0].runs[0].fileCount = 2;
    fs.writeFileSync(summaryPath, `${JSON.stringify(summary, null, 2)}\n`);
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /file count is inconsistent/);
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
      const registry = JSON.parse(fs.readFileSync(fixture.registryPath, "utf8"));
      registry.projects[0].typecheckPerformance[field] = value;
      fs.writeFileSync(fixture.registryPath, `${JSON.stringify(registry, null, 2)}\n`);
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
    ['process.stderr.write("error TS5058: missing config\\n"); process.exit(2);', /unparseable/],
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
