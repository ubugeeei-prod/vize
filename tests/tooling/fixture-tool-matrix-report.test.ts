import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { validatedFileCount } from "../../tools/fixtures/tool-matrix-metrics.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "fixtures", "tool-matrix-report.mjs");

function run(args: string[]) {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-matrix-"));
  const result = spawnSync(process.execPath, [toolPath, ...args, "--output-dir", outputDir], {
    cwd: root,
    encoding: "utf8",
  });
  return { outputDir, result };
}

function writeFakeVize(directory: string, body: string) {
  const executable = path.join(directory, "fake-vize.mjs");
  fs.writeFileSync(executable, `#!/usr/bin/env node\n${body}\n`);
  fs.chmodSync(executable, 0o755);
  return executable;
}

test("fixture tool matrix plans every registered project across all four required tools", () => {
  const { outputDir, result } = run(["--dry-run"]);
  try {
    assert.equal(result.status, 0, result.stderr);
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    assert.equal(report.schema, "vize.fixtureToolMatrixReport");
    assert.equal(report.version, 3);
    assert.match(report.evidence.commitSha, /^[0-9a-f]{40}$/);
    assert.deepEqual(Object.keys(report.evidence).sort(), ["commitSha", "machine", "runtime"]);
    assert.deepEqual(Object.keys(report.evidence.runtime).sort(), ["name", "version"]);
    assert.equal(report.evidence.runtime.name, "node");
    assert.equal(report.evidence.runtime.version, process.versions.node);
    assert.deepEqual(Object.keys(report.evidence.machine).sort(), [
      "arch",
      "cpuModel",
      "logicalCpuCount",
      "platform",
      "totalMemoryBytes",
    ]);
    assert.equal(report.evidence.machine.platform, process.platform);
    assert.equal(report.evidence.machine.arch, process.arch);
    assert.ok(report.evidence.machine.cpuModel.length > 0);
    assert.ok(report.evidence.machine.logicalCpuCount > 0);
    assert.ok(report.evidence.machine.totalMemoryBytes > 0);
    const markdown = fs.readFileSync(path.join(outputDir, "summary.md"), "utf8");
    assert.match(markdown, new RegExp(`Commit: ${report.evidence.commitSha}`));
    assert.match(markdown, /Runtime: node \d+\.\d+\.\d+/);
    assert.match(markdown, /Machine: [^/]+\/[^,]+, \d+ logical CPUs, \d+ bytes memory/);
    assert.match(markdown, /\bRequested\b/);
    assert.match(markdown, /\bTransitive Authored\b/);
    assert.match(markdown, /\bTransitive Dependencies\b/);
    assert.equal(report.summary.projectCount, 144);
    assert.equal(report.summary.toolCount, 4);
    assert.equal(report.summary.runCount, 576);
    assert.equal(report.summary.plannedRuns, 576);
    assert.equal(report.projects.length, 144);
    for (const project of report.projects) {
      assert.deepEqual(
        project.runs.map((entry: { tool: string }) => entry.tool),
        ["compiler", "typechecker", "linter", "formatter"],
        `${project.id} should exercise every requested tool`,
      );
      for (const entry of project.runs) {
        assert.equal(entry.fileCount, null);
        assert.equal(entry.coverage, null);
      }
    }
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects malformed commit evidence", () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-matrix-sha-"));
  try {
    const result = spawnSync(process.execPath, [toolPath, "--dry-run", "--output-dir", outputDir], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, GITHUB_SHA: "not-a-full-sha" },
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /GITHUB_SHA must be a full lowercase commit SHA/);
    assert.equal(fs.existsSync(path.join(outputDir, "summary.json")), false);
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix derives checked file evidence from validated tool payloads", () => {
  assert.equal(validatedFileCount("compiler", { compilerArtifacts: { inputFileCount: 7 } }), 7);
  assert.equal(validatedFileCount("typechecker", { parsed: { fileCount: 5 } }), 5);
  assert.equal(validatedFileCount("linter", { parsed: [{}, {}, {}] }), 3);
  assert.equal(validatedFileCount("formatter", { formatterCheck: { checkedFileCount: 11 } }), 11);
  assert.throws(
    () => validatedFileCount("unknown", {}),
    /Unsupported fixture matrix tool: unknown/,
  );
});

test("fixture tool matrix emits read-only commands with machine-readable diagnostics", () => {
  const { outputDir, result } = run([
    "--dry-run",
    "--project",
    "vue-vben-admin",
    "--tool",
    "compiler,typechecker,linter,formatter",
  ]);
  try {
    assert.equal(result.status, 0, result.stderr);
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    const runs = Object.fromEntries(
      report.projects[0].runs.map((entry: { tool: string }) => [entry.tool, entry]),
    ) as Record<string, { command: string }>;
    const compilerCommand = runs.compiler.command;
    assert.match(compilerCommand, /(?:^|\s)build(?:\s|$)/);
    assert.match(compilerCommand, /--format json(?:\s|$)/);
    assert.match(compilerCommand, /--output(?:\s|$)/);
    assert.equal(compilerCommand.includes("<compiler-output>"), true);
    assert.match(compilerCommand, /--template-syntax quirks(?:\s|$)/);
    assert.match(compilerCommand, /--continue-on-error(?:\s|$)/);
    assert.match(compilerCommand, /--no-config(?:\s|$)/);
    assert.match(
      runs.typechecker.command,
      /check playground\/src\/\*\*\/\*\.vue --format json --no-config --tsconfig playground\/tsconfig\.json/,
    );
    assert.doesNotMatch(runs.typechecker.command, /apps\/\*\*\/\*\.vue/);
    assert.match(runs.compiler.command, /apps\/\*\*\/\*\.vue/);
    assert.match(runs.linter.command, /lint .*--format json --preset ecosystem --no-config/);
    assert.match(runs.formatter.command, /fmt .*--check --no-config/);
    for (const entry of Object.values(runs)) {
      assert.doesNotMatch(entry.command, /(?:^|\s)--write(?:\s|$)/);
      assert.doesNotMatch(entry.command, /(?:^|\s)-w(?:\s|$)/);
    }
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects unknown projects, tools, and invalid timeouts", () => {
  for (const [args, message] of [
    [["--dry-run", "--project", "not-a-project"], /Unknown fixture project: not-a-project/],
    [["--dry-run", "--tool", "not-a-tool"], /Unknown fixture tool: not-a-tool/],
    [["--dry-run", "--timeout-ms", "0"], /--timeout-ms must be a positive integer/],
  ] as const) {
    const { outputDir, result } = run([...args]);
    try {
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
      assert.equal(fs.existsSync(path.join(outputDir, "summary.json")), false);
    } finally {
      fs.rmSync(outputDir, { recursive: true, force: true });
    }
  }
});

test("fixture tool matrix dry-run resolves a relative executable without invoking it", () => {
  const fakeDir = fs.mkdtempSync(path.join(root, "fixture-tool-matrix-probe-"));
  const marker = path.join(fakeDir, "invoked");
  const executable = writeFakeVize(
    fakeDir,
    `import fs from "node:fs"; fs.writeFileSync(${JSON.stringify(marker)}, "invoked");`,
  );
  const relativeExecutable = path.relative(root, executable);
  const { outputDir, result } = run([
    "--dry-run",
    "--project",
    "vue-vben-admin",
    "--tool",
    "compiler",
    "--vize-bin",
    relativeExecutable,
  ]);
  try {
    assert.equal(result.status, 0, result.stderr);
    assert.equal(fs.existsSync(marker), false, "dry-run must not probe the executable");
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    assert.equal(report.command.vize, executable);
    assert.match(report.projects[0].runs[0].command, new RegExp(`^${executable}`));
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix preserves raw output for failed invocations", () => {
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-failure-"));
  const executable = writeFakeVize(
    fakeDir,
    `if (process.argv[2] === "--version") process.exit(0);\nprocess.stdout.write("not json");\nprocess.stderr.write("synthetic failure");\nprocess.exit(2);`,
  );
  const { outputDir, result } = run([
    "--project",
    "vue-vben-admin",
    "--tool",
    "compiler",
    "--vize-bin",
    executable,
  ]);
  try {
    assert.equal(result.status, 1);
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    assert.equal(report.summary.failedRuns, 1);
    const rawPath = path.resolve(root, report.projects[0].runs[0].outputPath);
    assert.equal(fs.existsSync(rawPath), true, "failed run output must exist");
    const raw = JSON.parse(fs.readFileSync(rawPath, "utf8"));
    assert.equal(raw.exitCode, 2);
    assert.equal(raw.stdout, "not json");
    assert.equal(raw.stderr, "synthetic failure");
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix preserves raw output for invalid JSON and spawn errors", () => {
  const cases = [
    {
      name: "invalid-json",
      body: `if (process.argv[2] === "--version") process.exit(0);\nprocess.stdout.write("not json");`,
      expected: { exitCode: 0, stdout: "not json", parseError: true, spawnError: false },
    },
    {
      name: "spawn-error",
      body: `import fs from "node:fs"; import { fileURLToPath } from "node:url";\nif (process.argv[2] === "--version") { fs.unlinkSync(fileURLToPath(import.meta.url)); process.exit(0); }`,
      expected: { exitCode: null, stdout: "", parseError: false, spawnError: true },
    },
  ] as const;

  for (const fixtureCase of cases) {
    const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), `vize-${fixtureCase.name}-`));
    const executable = writeFakeVize(fakeDir, fixtureCase.body);
    const { outputDir, result } = run([
      "--project",
      "vue-vben-admin",
      "--tool",
      "linter",
      "--vize-bin",
      executable,
    ]);
    try {
      assert.equal(result.status, 1, fixtureCase.name);
      const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
      const rawPath = path.resolve(root, report.projects[0].runs[0].outputPath);
      const raw = JSON.parse(fs.readFileSync(rawPath, "utf8"));
      assert.equal(raw.exitCode, fixtureCase.expected.exitCode, fixtureCase.name);
      assert.equal(raw.stdout, fixtureCase.expected.stdout, fixtureCase.name);
      assert.equal("parseError" in raw, fixtureCase.expected.parseError, fixtureCase.name);
      assert.equal("spawnError" in raw, fixtureCase.expected.spawnError, fixtureCase.name);
    } finally {
      fs.rmSync(outputDir, { recursive: true, force: true });
      fs.rmSync(fakeDir, { recursive: true, force: true });
    }
  }
});

test("fixture tool matrix shards every project exactly once with balanced sizes", () => {
  const projectIds = new Set<string>();
  const shardSizes: number[] = [];
  for (let index = 0; index < 11; index += 1) {
    const { outputDir, result } = run([
      "--dry-run",
      "--shard-index",
      String(index),
      "--shard-count",
      "11",
    ]);
    try {
      assert.equal(result.status, 0, result.stderr);
      const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
      assert.equal(report.command.shardIndex, index);
      assert.equal(report.command.shardCount, 11);
      assert.equal(report.summary.runCount, report.summary.projectCount * 4);
      shardSizes.push(report.summary.projectCount);
      for (const project of report.projects) {
        assert.equal(
          projectIds.has(project.id),
          false,
          `${project.id} must appear in only one shard`,
        );
        projectIds.add(project.id);
      }
    } finally {
      fs.rmSync(outputDir, { recursive: true, force: true });
    }
  }
  assert.equal(projectIds.size, 144);
  assert.deepEqual(
    [...new Set(shardSizes)].sort((a, b) => a - b),
    [13, 14],
  );
  assert.equal(
    shardSizes.reduce((sum, size) => sum + size, 0),
    144,
  );
});

test("fixture tool matrix lists exactly the fixture paths selected by a shard", () => {
  const args = ["--shard-index", "3", "--shard-count", "11"];
  const planned = run(["--dry-run", ...args]);
  const listed = run(["--list-fixture-paths", ...args]);
  try {
    assert.equal(planned.result.status, 0, planned.result.stderr);
    assert.equal(listed.result.status, 0, listed.result.stderr);
    const report = JSON.parse(
      fs.readFileSync(path.join(planned.outputDir, "summary.json"), "utf8"),
    );
    assert.deepEqual(
      listed.result.stdout.trim().split("\n"),
      report.projects.map((project: { fixturePath: string }) => project.fixturePath),
    );
    assert.equal(fs.existsSync(path.join(listed.outputDir, "summary.json")), false);
  } finally {
    fs.rmSync(planned.outputDir, { recursive: true, force: true });
    fs.rmSync(listed.outputDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects invalid shard bounds", () => {
  for (const [args, message] of [
    [["--dry-run", "--shard-index", "-1"], /--shard-index must be a non-negative integer/],
    [["--dry-run", "--shard-count", "0"], /--shard-count must be a positive integer/],
    [
      ["--dry-run", "--shard-index", "2", "--shard-count", "2"],
      /--shard-index must be less than --shard-count/,
    ],
  ] as const) {
    const { outputDir, result } = run([...args]);
    try {
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
    } finally {
      fs.rmSync(outputDir, { recursive: true, force: true });
    }
  }
});
