#!/usr/bin/env node
/**
 * Fail-closed `vize check` benchmark gate (#3283).
 *
 * Publishes a timing artifact only after the measured binary reports every
 * planted diagnostic in the minimal plant projects and in a planted copy of
 * the timed corpus (tools/benchmarks/scripts/check-gate-plants.mjs). A missing native TypeScript
 * runtime, a missing binary, or a failed plant gate exits non-zero without
 * writing any timing, so a fast no-op can never rank. JS-engine (vue-tsc) and
 * native-TS-engine rows are reported as separate comparison classes; see
 * tools/benchmarks/scripts/check-gate-report.mjs for cold/steady separation and rotation.
 */

import os from "node:os";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  CORPUS_TSCONFIG,
  packageVersion,
  prepareCorpus,
  requireBinary,
  resolveOptionalVueTsc,
  resolveVuePackageDir,
  typescriptVersionNear,
} from "./check-gate-env.mjs";
import {
  countVueTscDiagnostics,
  gateVize,
  gateVueTsc,
  prepareCorpusPlant,
  prepareMinimalPlants,
} from "./check-gate-plants.mjs";
import { assertBinariesUnchanged, hashInPlace, pinExecutable } from "./benchmark-binary.mjs";
import { evaluateBudget, measureRows, renderMarkdown } from "./check-gate-report.mjs";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(benchDir, "..", "..", "..");

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    if (!argv[i].startsWith("--")) continue;
    const key = argv[i].slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

function parsePositiveInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function runCommand(binary, commandArgs, options = {}) {
  const start = performance.now();
  const result = spawnSync(binary, commandArgs, {
    cwd: options.cwd,
    env: { ...process.env, NO_COLOR: "1", VIZE_BENCH: "1", ...options.env },
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
  const ms = performance.now() - start;
  if (result.error) throw result.error;
  return { ms, status: result.status, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
}

function runVizeCheck(vizeBin, corsaPath, cwd, extraArgs = [], env = {}) {
  const args = ["check", ".", "--quiet", "--format", "json", "--tsconfig", "tsconfig.json"];
  const result = runCommand(vizeBin, [...args, "--corsa-path", corsaPath, ...extraArgs], {
    cwd,
    env,
  });
  let report = null;
  try {
    report = JSON.parse(result.stdout);
  } catch {
    report = null;
  }
  return { ...result, report };
}

function runVueTscCheck(vueTscBin, cwd) {
  return runCommand(vueTscBin, ["--noEmit", "-p", "tsconfig.json", "--pretty", "false"], { cwd });
}

export async function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const inputDir = resolve(args.input ?? join(benchDir, "__in__"));
  const runs = parsePositiveInt(args.runs, 3);
  const warmups = parsePositiveInt(args.warmups, 1);
  const fileCount = parsePositiveInt(args["check-file-count"], 500);
  const workRoot = resolve(args["work-root"] ?? join(rootDir, "target", "check-gate"));
  mkdirSync(workRoot, { recursive: true });

  const vizeSource = requireBinary("vize binary", args["vize-bin"], [
    join(rootDir, "target", "release", "vize"),
  ]);
  const binExt = process.platform === "win32" ? ".exe" : "";
  const tsgoSource = requireBinary("TypeScript 7/Corsa runtime", process.env.VIZE_CHECK_GATE_TSGO, [
    join(
      rootDir,
      "node_modules",
      "@typescript",
      `typescript-${process.platform}-${process.arch}`,
      "lib",
      `tsc${binExt}`,
    ),
    join(rootDir, "node_modules", ".bin", "tsgo"),
    join(rootDir, "tests", "node_modules", ".bin", "tsgo"),
  ]);
  // Measure a private copy of the vize binary: a shared CARGO_TARGET_DIR can be
  // rebuilt by another process mid-run, and a timing attributed to a binary
  // that no longer exists is worse than no timing. The native TypeScript
  // runtime is hashed in place so its package-relative lookup remains intact;
  // both are re-hashed before the artifact is written.
  const binaries = {
    vize: pinExecutable(vizeSource.path, workRoot),
    tsgo: hashInPlace(tsgoSource.path),
  };
  const vize = { ...vizeSource, path: binaries.vize.measuredPath };
  const tsgo = tsgoSource;
  const vuePackageDir = resolveVuePackageDir();
  if (!vuePackageDir) throw new Error("check-gate: vue package not found in any node_modules");
  const vueTsc = args["skip-vue-tsc"] === "true" ? null : resolveOptionalVueTsc();
  if (args["require-vue-tsc"] === "true" && !vueTsc) {
    throw new Error("check-gate: vue-tsc is required (--require-vue-tsc) but was not found");
  }

  const corpus = prepareCorpus(inputDir, fileCount, workRoot, vuePackageDir);
  const plants = prepareMinimalPlants(workRoot, vuePackageDir);
  const corpusPlant = prepareCorpusPlant(corpus.dir, CORPUS_TSCONFIG);
  // Un-timed reference runs: the generated corpus carries its own baseline
  // diagnostics, and the plant gate requires exactly baseline + plant.
  const vizeBaseline = runVizeCheck(vize.path, tsgo.path, corpus.dir);
  if (vizeBaseline.report == null) {
    throw new Error(
      `check-gate: vize produced no JSON report on the corpus.\n${vizeBaseline.stderr}`,
    );
  }
  const vueTscBaselineCount = vueTsc
    ? countVueTscDiagnostics(runVueTscCheck(vueTsc.path, corpus.dir).stdout)
    : null;
  let vizeReadiness;
  let vueTscGate = null;
  try {
    vizeReadiness = gateVize(
      (dir) => runVizeCheck(vize.path, tsgo.path, dir),
      plants.dirs,
      corpusPlant.dir,
      vizeBaseline.report,
    );
    if (vueTsc) {
      vueTscGate = gateVueTsc(
        (dir) => runVueTscCheck(vueTsc.path, dir),
        plants.dirs,
        corpusPlant.dir,
        vueTscBaselineCount,
      );
    }
  } finally {
    plants.cleanup();
    corpusPlant.cleanup();
  }

  const vizeDiagnostics = (out) =>
    out.report ? out.report.errorCount + out.report.warningCount : -1;
  const vizeExpected = vizeBaseline.report.errorCount + vizeBaseline.report.warningCount;
  const variants = [
    {
      id: "vize-check-1t",
      label: "Vize check (1T)",
      engineClass: "tsgo-native",
      expectedDiagnostics: vizeExpected,
      notes: "single Corsa server, RAYON_NUM_THREADS=1",
      countDiagnostics: vizeDiagnostics,
      measure: () =>
        runVizeCheck(vize.path, tsgo.path, corpus.dir, ["--servers", "1"], {
          RAYON_NUM_THREADS: "1",
        }),
    },
    {
      id: "vize-check-max",
      label: "Vize check (max)",
      engineClass: "tsgo-native",
      expectedDiagnostics: vizeExpected,
      notes: "auto-tuned Corsa sharding",
      countDiagnostics: vizeDiagnostics,
      measure: () => runVizeCheck(vize.path, tsgo.path, corpus.dir),
    },
  ];
  if (vueTsc && vueTscGate?.ok) {
    variants.unshift({
      id: "vue-tsc",
      label: "vue-tsc",
      engineClass: "typescript-js",
      expectedDiagnostics: vueTscBaselineCount,
      notes: "official Vue Language Tools CLI on the JS TypeScript engine",
      countDiagnostics: (out) => countVueTscDiagnostics(out.stdout),
      measure: () => runVueTscCheck(vueTsc.path, corpus.dir),
    });
  }

  const rows = measureRows(variants, { runs, warmups });
  // Nothing measured is attributable if a binary moved underneath the run.
  assertBinariesUnchanged(binaries);
  const budget = evaluateBudget(
    rows.find((row) => row.id === "vize-check-max").medianMs,
    args["budget-baseline"]
      ? JSON.parse(readFileSync(resolve(args["budget-baseline"]), "utf8"))
      : null,
    parsePositiveInt(args["budget-threshold"], 10),
  );

  const data = {
    schemaVersion: 1,
    kind: "vize-check-gate",
    generatedAt: new Date().toISOString(),
    commit: {
      sha: args.commit ?? process.env.GITHUB_SHA ?? "",
      ref: args.ref ?? process.env.GITHUB_REF_NAME ?? "",
      repository: args.repository ?? process.env.GITHUB_REPOSITORY ?? "",
    },
    runner: {
      label: args["runner-label"] ?? "local",
      cpuCount: os.cpus().length,
      cpuModel: os.cpus()[0]?.model ?? "unknown",
      platform: process.platform,
      arch: process.arch,
      node: process.version,
    },
    versions: {
      vize: vize.version,
      tsgo: tsgo.version,
      vueTsc: vueTsc?.version ?? null,
      typescript: vueTsc ? typescriptVersionNear(vueTsc.path) : null,
      vue: packageVersion(vuePackageDir),
    },
    binaries,
    entry: {
      tsconfigPath: corpus.tsconfigPath,
      corpusDir: corpus.dir,
      fileCount: corpus.files.length,
      totalBytes: corpus.totalBytes,
    },
    backend: { corsaPath: tsgo.path, vize: vizeReadiness, vueTsc: vueTscGate?.readiness ?? null },
    settings: { runs, warmups },
    skipped: {
      "typescript-js": vueTsc
        ? vueTscGate?.ok
          ? null
          : "vue-tsc failed the plant gate; row unranked"
        : "vue-tsc missing or skipped",
    },
    rows,
    budget,
  };

  const markdown = renderMarkdown(data);
  if (args.out) writeFileSync(resolve(args.out), markdown);
  else process.stdout.write(markdown);
  if (args.json) writeFileSync(resolve(args.json), `${JSON.stringify(data, null, 2)}\n`);
  if (
    budget.status === "failed" ||
    budget.status === "invalid-baseline" ||
    budget.status === "invalid-head-median"
  ) {
    throw new Error(`check-gate: budget ${budget.status} (${JSON.stringify(budget)})`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
