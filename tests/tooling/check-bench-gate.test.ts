import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// @ts-expect-error plain-JS bench module without type declarations
import { generateCorpus } from "../../bench/generate.mjs";
// @ts-expect-error plain-JS bench module without type declarations
import { evaluateBudget } from "../../bench/check-gate-report.mjs";
import { resolveVizeCommand } from "../_helpers/realworld-typecheck.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const gateScript = path.join(root, "bench/check-gate.mjs");
const outputRoot = path.join(root, "target/vize-tests/check-bench-gate");

type GateRun = {
  status: number | null;
  stderr: string;
  stdout: string;
};

function runGate(args: string[], env: Record<string, string> = {}): GateRun {
  const result = spawnSync(process.execPath, [gateScript, ...args], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...env },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 600_000,
  });
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, stdout: result.stdout };
}

function resolveTsgoBinary(): string | undefined {
  const candidates = [
    path.join(root, "node_modules/.bin/tsgo"),
    path.join(root, "tests/node_modules/.bin/tsgo"),
  ];
  return candidates.find((candidate) => fs.existsSync(candidate));
}

function resolveVizeBinary(): string | undefined {
  const [command, ...prefix] = resolveVizeCommand();
  return prefix.length === 0 && path.isAbsolute(command) ? command : undefined;
}

test("check-gate budget rule is pure and strict", () => {
  assert.deepEqual(evaluateBudget(100, null, 10), { status: "no-baseline", thresholdPercent: 10 });
  assert.deepEqual(evaluateBudget(100, { rows: [] }, 10), {
    status: "invalid-baseline",
    thresholdPercent: 10,
  });
  const baseline = { rows: [{ id: "vize-check-max", medianMs: 100 }] };
  assert.deepEqual(evaluateBudget(109.9, baseline, 10), {
    status: "passed",
    baseMedianMs: 100,
    headMedianMs: 109.9,
    changePercent: 9.9,
    thresholdPercent: 10,
  });
  assert.deepEqual(evaluateBudget(110, baseline, 10), {
    status: "failed",
    baseMedianMs: 100,
    headMedianMs: 110,
    changePercent: 10,
    thresholdPercent: 10,
  });
});

test("check-gate fails closed when the pinned tsgo binary is missing", () => {
  fs.mkdirSync(outputRoot, { recursive: true });
  const workDir = fs.mkdtempSync(path.join(outputRoot, "missing-tsgo-"));
  const jsonPath = path.join(workDir, "results.json");
  try {
    const run = runGate(
      ["--vize-bin", "target/ci/vize", "--json", jsonPath, "--work-root", workDir],
      { VIZE_CHECK_GATE_TSGO: path.join(workDir, "does-not-exist") },
    );
    assert.equal(run.status, 1, run.stderr || run.stdout);
    assert.match(run.stderr, /check-gate: tsgo binary not found/);
    assert.equal(fs.existsSync(jsonPath), false, "no timing artifact may be written");
  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
  }
});

test("check-gate refuses to time a binary that misses the plants", (t) => {
  const tsgo = resolveTsgoBinary();
  if (tsgo == null) {
    requireTsgoOrSkip(t);
    return;
  }
  fs.mkdirSync(outputRoot, { recursive: true });
  const workDir = fs.mkdtempSync(path.join(outputRoot, "fake-vize-"));
  const jsonPath = path.join(workDir, "results.json");
  const fakeVize = path.join(workDir, "fake-vize.sh");
  // Answers --version, then reports a clean project for every check: the
  // canonical fast no-op that must never rank.
  fs.writeFileSync(
    fakeVize,
    `#!/bin/sh
if [ "$1" = "--version" ]; then echo "vize 0.0.0-fake"; exit 0; fi
echo '{"files":[],"errorCount":0,"warningCount":0,"fileCount":0}'
exit 0
`,
  );
  fs.chmodSync(fakeVize, 0o755);
  const inputDir = path.join(workDir, "in");
  generateCorpus({ fileCount: 2, benchDir: inputDir, log: () => {} });
  try {
    const run = runGate([
      "--vize-bin",
      fakeVize,
      "--input",
      inputDir,
      "--json",
      jsonPath,
      "--work-root",
      workDir,
      "--skip-vue-tsc",
    ]);
    assert.equal(run.status, 1, run.stderr || run.stdout);
    assert.match(run.stderr, /vize produced no JSON report|missed the script assignment plant/);
    assert.equal(fs.existsSync(jsonPath), false, "no timing artifact may be written");
  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
  }
});

test("check-gate publishes reproducibility metadata for a gated run", (t) => {
  const tsgo = resolveTsgoBinary();
  const vizeBin = resolveVizeBinary();
  if (tsgo == null || vizeBin == null) {
    requireTsgoOrSkip(t);
    return;
  }
  fs.mkdirSync(outputRoot, { recursive: true });
  const workDir = fs.mkdtempSync(path.join(outputRoot, "real-run-"));
  const inputDir = path.join(workDir, "in");
  const jsonPath = path.join(workDir, "results.json");
  const markdownPath = path.join(workDir, "results.md");
  generateCorpus({ fileCount: 6, benchDir: inputDir, log: () => {} });
  try {
    const run = runGate([
      "--vize-bin",
      vizeBin,
      "--input",
      inputDir,
      "--check-file-count",
      "6",
      "--runs",
      "1",
      "--warmups",
      "1",
      "--skip-vue-tsc",
      "--json",
      jsonPath,
      "--out",
      markdownPath,
      "--work-root",
      workDir,
    ]);
    assert.equal(run.status, 0, run.stderr || run.stdout);

    const data = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
    assert.equal(data.kind, "vize-check-gate");
    const vizeVersion = spawnSync(vizeBin, ["--version"], { encoding: "utf8" }).stdout.trim();
    assert.equal(data.versions.vize, vizeVersion);
    assert.match(data.versions.tsgo, /\d/);
    assert.match(data.versions.vue, /^\d/);
    assert.deepEqual(data.backend.vize, {
      script: true,
      templateProp: true,
      templateEvent: true,
      componentProp: true,
      corpus: true,
    });
    assert.equal(data.entry.fileCount, 6);
    assert.ok(data.entry.totalBytes > 0);
    assert.ok(data.entry.tsconfigPath.endsWith("tsconfig.json"));
    assert.deepEqual(data.settings, { runs: 1, warmups: 1 });
    assert.equal(data.skipped["typescript-js"], "vue-tsc missing or skipped");
    assert.equal(data.budget.status, "no-baseline");

    assert.deepEqual(
      data.rows.map((row: { id: string }) => row.id),
      ["vize-check-1t", "vize-check-max"],
    );
    for (const row of data.rows) {
      assert.equal(row.engineClass, "tsgo-native");
      assert.equal(row.status, "ok");
      assert.ok(row.coldMs > 0, "cold startup must be recorded separately");
      assert.equal(row.runs.length, 1);
      assert.equal(row.medianMs, row.runs[0]);
      assert.equal(typeof row.diagnosticCount, "number");
      assert.equal(row.warmupPasses, 1);
    }

    const markdown = fs.readFileSync(markdownPath, "utf8");
    assert.match(markdown, /Backend readiness .*corpus=pass/);
    assert.match(markdown, /### native TypeScript engine \(tsgo\)/);
    assert.match(markdown, /### JS TypeScript engine \(tsc\)/);
    assert.match(markdown, /ranked separately/);
  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
  }
});

function requireTsgoOrSkip(t: { skip(reason: string): void }): void {
  assert.notEqual(
    process.env.VIZE_TEST_REQUIRE_TSGO,
    "1",
    "tsgo (or a built vize binary) is required but unavailable",
  );
  t.skip("tsgo or a built vize binary is unavailable");
}
