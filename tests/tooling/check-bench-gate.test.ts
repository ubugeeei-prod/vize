import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// @ts-expect-error plain-JS bench module without type declarations
import { generateCorpus } from "../../bench/generate.mjs";
// @ts-expect-error plain-JS bench module without type declarations
import { evaluateBudget } from "../../bench/check-gate-report.mjs";
import { resolveVizeCommand } from "../_helpers/realworld-typecheck.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

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
  return resolveTypecheckRuntime(root);
}

function sha256Of(filePath: string): string {
  return createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function readVueVersion(): string {
  const candidates = [
    path.join(root, "bench/node_modules/vue/package.json"),
    path.join(root, "node_modules/vue/package.json"),
    path.join(root, "tests/node_modules/vue/package.json"),
  ];
  const found = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(found, "the vue package must be installed for the check gate");
  return JSON.parse(fs.readFileSync(found, "utf8")).version;
}

function totalVueBytes(dir: string): number {
  return fs
    .readdirSync(dir)
    .filter((file) => file.endsWith(".vue"))
    .reduce((sum, file) => sum + fs.statSync(path.join(dir, file)).size, 0);
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

test("check-gate fails closed when the pinned TypeScript 7/Corsa runtime is missing", (t) => {
  // Resolve the vize binary rather than hardcoding target/ci/vize: a worktree
  // with a shared CARGO_TARGET_DIR has no such path, and this test is about
  // the runtime failure, not about where vize was built.
  const vizeBin = resolveVizeBinary();
  if (vizeBin == null) {
    requireRuntimeOrSkip(t);
    return;
  }
  fs.mkdirSync(outputRoot, { recursive: true });
  const workDir = fs.mkdtempSync(path.join(outputRoot, "missing-tsgo-"));
  const jsonPath = path.join(workDir, "results.json");
  try {
    const run = runGate(["--vize-bin", vizeBin, "--json", jsonPath, "--work-root", workDir], {
      VIZE_CHECK_GATE_TSGO: path.join(workDir, "does-not-exist"),
    });
    assert.equal(run.status, 1, run.stderr || run.stdout);
    assert.match(run.stderr, /check-gate: TypeScript 7\/Corsa runtime not found/);
    assert.equal(fs.existsSync(jsonPath), false, "no timing artifact may be written");
  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
  }
});

test("check-gate refuses to time a binary that misses the plants", (t) => {
  const tsgo = resolveTsgoBinary();
  if (tsgo == null) {
    requireRuntimeOrSkip(t);
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
    requireRuntimeOrSkip(t);
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

    // The whole artifact shape, not a sample of it: a reader must be able to
    // reproduce the number from the artifact alone, so a silently dropped
    // metadata block has to fail this gate.
    assert.deepEqual(Object.keys(data).sort(), [
      "backend",
      "binaries",
      "budget",
      "commit",
      "entry",
      "generatedAt",
      "kind",
      "rows",
      "runner",
      "schemaVersion",
      "settings",
      "skipped",
      "versions",
    ]);
    assert.equal(data.schemaVersion, 1);
    assert.equal(data.kind, "vize-check-gate");

    assert.deepEqual(data.versions, {
      vize: spawnSync(vizeBin, ["--version"], { encoding: "utf8" }).stdout.trim(),
      tsgo: spawnSync(tsgo, ["--version"], { encoding: "utf8" }).stdout.trim().split("\n")[0],
      vueTsc: null,
      typescript: null,
      vue: readVueVersion(),
    });

    assert.deepEqual(data.backend, {
      corsaPath: path.resolve(tsgo),
      vize: {
        script: true,
        templateProp: true,
        templateEvent: true,
        componentProp: true,
        corpus: true,
      },
      vueTsc: null,
    });

    // The measured vize is a private copy pinned by content hash, so a rebuild
    // of the source path cannot change what produced these timings.
    const vizeSha = sha256Of(vizeBin);
    assert.deepEqual(data.binaries, {
      vize: {
        source: path.resolve(vizeBin),
        measuredPath: path.join(workDir, "pinned-binaries", `vize-${vizeSha.slice(0, 16)}`),
        sha256: vizeSha,
        pinned: true,
      },
      tsgo: {
        source: path.resolve(tsgo),
        measuredPath: path.resolve(tsgo),
        sha256: sha256Of(tsgo),
        pinned: false,
      },
    });
    assert.equal(fs.existsSync(data.binaries.vize.measuredPath), true);

    const corpusDir = path.join(workDir, "corpus-6");
    assert.deepEqual(data.entry, {
      tsconfigPath: path.join(corpusDir, "tsconfig.json"),
      corpusDir,
      fileCount: 6,
      totalBytes: totalVueBytes(inputDir),
    });
    assert.deepEqual(data.settings, { runs: 1, warmups: 1 });
    assert.deepEqual(data.skipped, { "typescript-js": "vue-tsc missing or skipped" });
    assert.deepEqual(data.budget, { status: "no-baseline", thresholdPercent: 10 });

    assert.deepEqual(
      data.rows.map((row: Record<string, unknown>) => ({
        ...row,
        coldMs: typeof row.coldMs === "number" && row.coldMs > 0,
        runs: (row.runs as number[]).length,
        medianMs: row.medianMs === (row.runs as number[])[0],
        diagnosticCount: typeof row.diagnosticCount,
      })),
      [
        {
          id: "vize-check-1t",
          label: "Vize check (1T)",
          engineClass: "tsgo-native",
          status: "ok",
          coldMs: true,
          runs: 1,
          medianMs: true,
          diagnosticCount: "number",
          warmupPasses: 1,
          notes: "single Corsa server, RAYON_NUM_THREADS=1",
        },
        {
          id: "vize-check-max",
          label: "Vize check (max)",
          engineClass: "tsgo-native",
          status: "ok",
          coldMs: true,
          runs: 1,
          medianMs: true,
          diagnosticCount: "number",
          warmupPasses: 1,
          notes: "auto-tuned Corsa sharding",
        },
      ],
    );

    // Every non-timing line of the report, in order.
    const markdown = fs.readFileSync(markdownPath, "utf8").split("\n");
    assert.deepEqual(
      markdown.filter((line) => !line.startsWith("| Vize check")),
      [
        "## Vize Check Benchmark Gate",
        "",
        `Measured: ${data.generatedAt}`,
        `Versions: \`${data.versions.vize}\` · tsgo \`${data.versions.tsgo}\` · vue-tsc \`missing\` (typescript \`n/a\`) · vue \`${data.versions.vue}\``,
        `Binaries (sha256 of the measured file, re-checked after the run): vize=\`${data.binaries.vize.sha256}\` tsgo=\`${data.binaries.tsgo.sha256}\``,
        `Entry point: \`${data.entry.tsconfigPath}\` — 6 unique SFC files, ${data.entry.totalBytes.toLocaleString("en-US")} bytes.`,
        "Backend readiness (planted-diagnostic gates, all required before timing): script=pass templateProp=pass templateEvent=pass componentProp=pass corpus=pass",
        "Budget: no-baseline",
        "",
        "### JS TypeScript engine (tsc)",
        "",
        "| Row | Cold start | Warmed median | Diagnostics | Measured runs |",
        "| --- | ---: | ---: | ---: | --- |",
        "| (vue-tsc missing or skipped) | n/a | n/a | n/a | n/a |",
        "",
        "### native TypeScript engine (tsgo)",
        "",
        "| Row | Cold start | Warmed median | Diagnostics | Measured runs |",
        "| --- | ---: | ---: | ---: | --- |",
        "",
        "Engine classes are ranked separately: a cross-class ratio measures TypeScript's native rewrite as much as the Vue layer, so it is reported as context only.",
        "",
      ],
    );
  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
  }
});

function requireRuntimeOrSkip(t: { skip(reason: string): void }): void {
  requireTypecheckDependency(
    t,
    undefined,
    "TypeScript 7/Corsa runtime (or a built vize binary) for the check benchmark gate",
    "TypeScript 7/Corsa runtime or a built vize binary is unavailable",
  );
}
