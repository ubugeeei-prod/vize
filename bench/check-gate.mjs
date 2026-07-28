#!/usr/bin/env node
/**
 * Fail-closed `vize check` benchmark gate (#3283).
 *
 * Publishes a timing artifact only after the measured binary reports every
 * planted diagnostic in the minimal plant projects and in a planted copy of
 * the timed corpus (bench/check-gate-plants.mjs). A missing tsgo, a missing
 * binary, or a failed plant gate exits non-zero without writing any timing,
 * so a fast no-op can never rank. JS-engine (vue-tsc) and native-TS-engine
 * (vize + tsgo) rows are reported as separate comparison classes; see
 * bench/check-gate-report.mjs for cold/steady separation and rotation.
 */

import { createRequire } from "node:module";
import os from "node:os";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  countVueTscDiagnostics,
  gateVize,
  gateVueTsc,
  prepareCorpusPlant,
  prepareMinimalPlants,
} from "./check-gate-plants.mjs";
import { evaluateBudget, measureRows, renderMarkdown } from "./check-gate-report.mjs";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = dirname(benchDir);

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
    env: { ...process.env, NO_COLOR: "1", VIZE_BENCH: "1", ...(options.env ?? {}) },
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
  const ms = performance.now() - start;
  if (result.error) throw result.error;
  return { ms, status: result.status, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
}

/**
 * Resolve a binary that MUST exist and answer --version; fail closed. An
 * explicit pin is exclusive: a benchmark asked to measure one binary must
 * never silently fall back to another.
 */
function requireBinary(label, explicitPath, fallbacks) {
  const candidates = (explicitPath ? [explicitPath] : fallbacks).map((c) => resolve(c));
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) {
    throw new Error(`check-gate: ${label} not found (looked at: ${candidates.join(", ")})`);
  }
  const probe = spawnSync(found, ["--version"], { encoding: "utf8" });
  if (probe.status !== 0) {
    throw new Error(`check-gate: ${label} at ${found} failed --version`);
  }
  return { path: found, version: (probe.stdout || probe.stderr).trim().split("\n")[0] };
}

function resolveOptionalVueTsc() {
  const candidates = [
    process.env.VIZE_CHECK_GATE_VUE_TSC,
    join(benchDir, "node_modules", ".bin", "vue-tsc"),
    join(rootDir, "node_modules", ".bin", "vue-tsc"),
    join(rootDir, "tests", "node_modules", ".bin", "vue-tsc"),
  ].filter(Boolean);
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) return null;
  const probe = spawnSync(found, ["--version"], { encoding: "utf8" });
  if (probe.status !== 0) return null;
  return { path: found, version: (probe.stdout || "").trim().split("\n")[0] };
}

function resolveVuePackageDir() {
  const candidates = [
    join(benchDir, "node_modules", "vue"),
    join(rootDir, "node_modules", "vue"),
    join(rootDir, "tests", "node_modules", "vue"),
  ];
  return candidates.find((candidate) => existsSync(candidate)) ?? null;
}

function packageVersion(packageDir) {
  try {
    return JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8")).version ?? null;
  } catch {
    return null;
  }
}

function typescriptVersionNear(vueTscPath) {
  try {
    const require = createRequire(join(dirname(realpathSync(vueTscPath)), "package.json"));
    return JSON.parse(readFileSync(require.resolve("typescript/package.json"), "utf8")).version;
  } catch {
    return null;
  }
}

const CORPUS_TSCONFIG = {
  compilerOptions: {
    esModuleInterop: true,
    isolatedModules: true,
    lib: ["ESNext", "DOM"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ESNext",
    types: [],
  },
  vueCompilerOptions: { strictTemplates: true },
  include: ["*.vue"],
};

/** Copy the measured corpus subset into a self-contained project dir. */
function prepareCorpus(inputDir, fileCount, workRoot, vuePackageDir) {
  if (!existsSync(inputDir)) {
    throw new Error(`check-gate: input corpus not found: ${inputDir} (run bench/generate.mjs first)`);
  }
  const files = readdirSync(inputDir)
    .filter((file) => file.endsWith(".vue"))
    .sort()
    .slice(0, fileCount);
  if (files.length === 0) throw new Error(`check-gate: no .vue files found in ${inputDir}`);
  const dir = join(workRoot, `corpus-${files.length}`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  let totalBytes = 0;
  for (const file of files) {
    copyFileSync(join(inputDir, file), join(dir, file));
    totalBytes += statSync(join(dir, file)).size;
  }
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(CORPUS_TSCONFIG, null, 2)}\n`);
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify({ name: "vize-check-gate-corpus", private: true, type: "module" }, null, 2)}\n`,
  );
  const nodeModules = join(dir, "node_modules");
  mkdirSync(nodeModules, { recursive: true });
  symlinkSync(vuePackageDir, join(nodeModules, "vue"), "dir");
  const vueNamespace = join(dirname(vuePackageDir), "@vue");
  if (existsSync(vueNamespace)) symlinkSync(vueNamespace, join(nodeModules, "@vue"), "dir");
  return { dir, files, totalBytes, tsconfigPath: join(dir, "tsconfig.json") };
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

  const vize = requireBinary("vize binary", args["vize-bin"], [
    join(rootDir, "target", "release", "vize"),
  ]);
  const tsgo = requireBinary("tsgo binary", process.env.VIZE_CHECK_GATE_TSGO, [
    join(rootDir, "node_modules", ".bin", "tsgo"),
    join(rootDir, "tests", "node_modules", ".bin", "tsgo"),
  ]);
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
    throw new Error(`check-gate: vize produced no JSON report on the corpus.\n${vizeBaseline.stderr}`);
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
  if (budget.status === "failed" || budget.status === "invalid-baseline") {
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
