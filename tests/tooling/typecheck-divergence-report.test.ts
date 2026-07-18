import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
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
  writeJson(registryPath, { projects: [project] });
  const outputPath = path.join(reportDir, "fixture-typechecker.json");
  const parsed = {
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
    files: [{ file: "src/App.vue", diagnostics: ["error:1:1 [TS2322] shared"] }],
  };
  writeJson(outputPath, {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: "fixture",
    tool: "typechecker",
    exitCode: 1,
    stdout: JSON.stringify(parsed),
    stderr: "",
    parsed,
  });
  writeJson(path.join(reportDir, "summary.json"), {
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
            outputPath: path.relative(root, outputPath),
          },
        ],
      },
    ],
  });
  const vueTsc = path.join(fakeDir, "vue-tsc.mjs");
  const invocationPath = path.join(fakeDir, "invocation.json");
  writeVueTsc(
    vueTsc,
    'process.stdout.write("src/App.vue(1,1): error TS2322: shared\\n"); process.exit(2);',
    invocationPath,
  );
  return { fixtureRoot, reportDir, fakeDir, registryPath, outputPath, vueTsc, invocationPath };
}

function writeVueTsc(pathname: string, runBody: string, invocationPath?: string) {
  const recordInvocation =
    invocationPath == null
      ? ""
      : `fs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ cwd: process.cwd(), args: process.argv.slice(2) }));`;
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node\nimport fs from "node:fs";\nif (process.argv.includes("--version")) { console.log("3.3.4"); process.exit(0); }\n${recordInvocation}\n${runBody}\n`,
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

function readJson(pathname: string) {
  return JSON.parse(fs.readFileSync(pathname, "utf8"));
}

function writeJson(pathname: string, value: unknown) {
  fs.writeFileSync(pathname, `${JSON.stringify(value, null, 2)}\n`);
}

function updateJson(pathname: string, update: (value: any) => void) {
  const value = readJson(pathname);
  update(value);
  writeJson(pathname, value);
}

test("typecheck divergence report binds baseline evidence to the matrix artifact", () => {
  const fixture = setup();
  try {
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
    assert.equal(artifact.evidence.commitSha, commitSha);
    assert.deepEqual(artifact.source, {
      payloadSha256: createHash("sha256").update(fs.readFileSync(fixture.outputPath)).digest("hex"),
      fileCount: 1,
    });
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
    assert.deepEqual(readJson(fixture.invocationPath), {
      cwd: fixture.fixtureRoot,
      args: ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
    });
    const markdown = fs.readFileSync(
      path.join(fixture.reportDir, "fixture-typecheck-divergence.md"),
      "utf8",
    );
    assert.match(markdown, /Budget passed: true/);
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
