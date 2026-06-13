#!/usr/bin/env node
/**
 * Dialect guard: prove that the opt-in `legacy` Vue (v0/v1/v2) cargo feature is
 * truly zero-impact on the default Vue 3 path.
 *
 * Two independent checks run against the same generated Vue 3 corpus:
 *
 *   1. Codegen identity — compile the corpus once with a `legacy`-OFF binary and
 *      once with a `legacy`-ON binary and assert the emitted output is
 *      byte-identical. Legacy support is dialect-gated, so with a Vue 3 corpus
 *      the ON binary must produce exactly the Vue 3 output; any divergence means
 *      legacy code leaked into the default path.
 *
 *   2. Paired A/B timing — time the OFF vs ON binary on the corpus in
 *      alternating pairs and fail if the ON build regresses the Vue 3 hot path
 *      past `--threshold` (default 2%) and the absolute paired delta is at
 *      least `--min-regression-ms` (default 5ms). The feature is meant to add
 *      code behind a flag, not slow the default binary.
 *
 * HONEST STATUS: the `legacy` feature is still largely a stub today, so the
 * codegen-identity assertion is currently expected to pass trivially and the A/B
 * delta is near zero. This harness exists so the gate is wired and starts
 * catching divergence the moment `legacy` grows real, dialect-gated code. See
 * the PR's "Deferred" section.
 *
 * Dependency-free besides `cargo` and a built corpus generator.
 */

import { spawnSync } from "node:child_process";
import { appendFileSync, existsSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

const DEFAULT_THRESHOLD_PERCENT = 2;
const DEFAULT_MIN_REGRESSION_MS = 5;
const DEFAULT_RUNS = 5;
const DEFAULT_WARMUPS = 1;

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      continue;
    }
    const key = arg.slice(2);
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

function requireArg(args, key) {
  const value = args[key];
  if (!value) {
    throw new Error(`Missing required argument: --${key}`);
  }
  return value;
}

function parsePositiveInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function parseNonNegativeInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
}

function parsePositiveFloat(value, fallback) {
  const parsed = Number.parseFloat(value ?? "");
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function parseNonNegativeFloat(value, fallback) {
  const parsed = Number.parseFloat(value ?? "");
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) {
    return sorted[mid];
  }
  return (sorted[mid - 1] + sorted[mid]) / 2;
}

/**
 * Hash every emitted file under a build output directory so two builds can be
 * compared independent of filesystem walk order. Returns a map of relative
 * path -> sha256, and a combined digest over the sorted entries.
 */
export function hashOutputDir(dir) {
  const files = {};
  const walk = (current, prefix) => {
    for (const entry of readdirSync(current, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name),
    )) {
      const full = join(current, entry.name);
      const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        walk(full, rel);
      } else if (entry.isFile()) {
        files[rel] = createHash("sha256").update(readFileSync(full)).digest("hex");
      }
    }
  };
  if (existsSync(dir)) {
    walk(dir, "");
  }
  const combined = createHash("sha256");
  for (const rel of Object.keys(files).sort()) {
    combined.update(rel).update("\0").update(files[rel]).update("\0");
  }
  return { files, digest: combined.digest("hex") };
}

/**
 * Diff two output-directory hash maps. Returns the list of paths that are
 * missing on one side or whose contents differ.
 */
export function diffOutputs(off, on) {
  const differences = [];
  const names = new Set([...Object.keys(off.files), ...Object.keys(on.files)]);
  for (const name of [...names].sort()) {
    const offHash = off.files[name];
    const onHash = on.files[name];
    if (offHash == null) {
      differences.push(`${name}: only emitted with legacy ON`);
    } else if (onHash == null) {
      differences.push(`${name}: only emitted with legacy OFF`);
    } else if (offHash !== onHash) {
      differences.push(`${name}: byte content differs between legacy OFF and ON`);
    }
  }
  return differences;
}

function run(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    cwd: options.cwd,
    env: { ...process.env, ...options.env },
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0 && !options.allowFailure) {
    throw new Error(`${command} ${commandArgs.join(" ")} exited with ${result.status}`);
  }
  return result;
}

function buildArgs(inputDir, outDir) {
  // Deterministic, single-threaded JS emit so the OFF/ON outputs are comparable
  // and the timing lane is stable on shared runners.
  return [
    "build",
    inputDir,
    "--output",
    outDir,
    "--format",
    "js",
    "--threads",
    "1",
    "--continue-on-error",
  ];
}

function compileCorpus(bin, inputDir, outDir) {
  rmSync(outDir, { recursive: true, force: true });
  run(bin, buildArgs(inputDir, outDir), {
    env: { NO_COLOR: "1", RAYON_NUM_THREADS: "1", VIZE_BENCH: "1" },
  });
}

function timeCompile(bin, inputDir, outDir) {
  const start = performance.now();
  compileCorpus(bin, inputDir, outDir);
  return performance.now() - start;
}

function timeCorpusPair(offBin, onBin, inputDir, { offOut, onOut, runs, warmups }) {
  const offSamples = [];
  const onSamples = [];
  const ratios = [];
  const deltas = [];

  for (let i = 0; i < warmups; i++) {
    if (i % 2 === 0) {
      compileCorpus(offBin, inputDir, offOut);
      compileCorpus(onBin, inputDir, onOut);
    } else {
      compileCorpus(onBin, inputDir, onOut);
      compileCorpus(offBin, inputDir, offOut);
    }
  }

  for (let i = 0; i < runs; i++) {
    let offMs;
    let onMs;
    if (i % 2 === 0) {
      offMs = timeCompile(offBin, inputDir, offOut);
      onMs = timeCompile(onBin, inputDir, onOut);
    } else {
      onMs = timeCompile(onBin, inputDir, onOut);
      offMs = timeCompile(offBin, inputDir, offOut);
    }
    offSamples.push(offMs);
    onSamples.push(onMs);
    if (offMs > 0) {
      ratios.push(onMs / offMs);
    }
    deltas.push(onMs - offMs);
  }

  return {
    offSamples,
    onSamples,
    ratio: ratios.length > 0 ? median(ratios) : Number.NaN,
    deltaMs: deltas.length > 0 ? median(deltas) : Number.NaN,
  };
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const inputDir = resolve(requireArg(args, "input"));
  const offBin = resolve(requireArg(args, "off-bin"));
  const onBin = resolve(requireArg(args, "on-bin"));
  const outRoot = resolve(args["out-dir"] ?? "dialect-guard-out");
  const threshold = parsePositiveFloat(args.threshold, DEFAULT_THRESHOLD_PERCENT);
  const minRegressionMs = parseNonNegativeFloat(
    args["min-regression-ms"],
    DEFAULT_MIN_REGRESSION_MS,
  );
  const runs = parsePositiveInt(args.runs, DEFAULT_RUNS);
  const warmups = parseNonNegativeInt(args.warmups, DEFAULT_WARMUPS);

  if (!existsSync(inputDir)) {
    throw new Error(`Input directory not found: ${inputDir}`);
  }
  if (!existsSync(offBin)) {
    throw new Error(`legacy-OFF binary not found: ${offBin}`);
  }
  if (!existsSync(onBin)) {
    throw new Error(`legacy-ON binary not found: ${onBin}`);
  }

  const offOut = join(outRoot, "off");
  const onOut = join(outRoot, "on");

  // --- 1. Codegen identity -------------------------------------------------
  compileCorpus(offBin, inputDir, offOut);
  compileCorpus(onBin, inputDir, onOut);
  const offHash = hashOutputDir(offOut);
  const onHash = hashOutputDir(onOut);
  const differences = diffOutputs(offHash, onHash);
  const identical = differences.length === 0;

  // --- 2. A/B timing -------------------------------------------------------
  const { offSamples, onSamples, ratio, deltaMs } = timeCorpusPair(offBin, onBin, inputDir, {
    offOut,
    onOut,
    runs,
    warmups,
  });
  const offMs = median(offSamples);
  const onMs = median(onSamples);
  const changePercent = Number.isFinite(ratio) ? (ratio - 1) * 100 : Number.NaN;
  const regressed =
    Number.isFinite(changePercent) &&
    Number.isFinite(deltaMs) &&
    changePercent >= threshold &&
    deltaMs >= minRegressionMs;

  const lines = [];
  lines.push("## Dialect Guard (legacy feature)");
  lines.push("");
  lines.push(
    `Vue 3 corpus compiled with the \`legacy\` feature OFF vs ON. Codegen must stay byte-identical and the default path must not regress past ${threshold}% with an absolute paired delta of ${minRegressionMs}ms or more.`,
  );
  lines.push("");
  lines.push("| Check | Result |");
  lines.push("| --- | --- |");
  const identityCell = identical ? "identical" : `${differences.length} diff(s)`;
  const offDigest = offHash.digest.slice(0, 12);
  lines.push(`| Codegen identity (OFF digest \`${offDigest}\`) | ${identityCell} |`);
  const deltaCell = Number.isFinite(changePercent)
    ? `${changePercent >= 0 ? "+" : ""}${changePercent.toFixed(2)}% / ${deltaMs >= 0 ? "+" : ""}${deltaMs.toFixed(1)}ms`
    : "n/a";
  const timingLabel = `Paired A/B timing (OFF median ${offMs.toFixed(1)}ms to ON median ${onMs.toFixed(1)}ms)`;
  lines.push(`| ${timingLabel} | ${deltaCell} ${regressed ? "regression" : "ok"} |`);
  if (!identical) {
    lines.push("");
    lines.push("Codegen divergences:");
    for (const difference of differences.slice(0, 20)) {
      lines.push(`- ${difference}`);
    }
  }
  lines.push("");
  const summary = `${lines.join("\n")}\n`;

  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
  } else {
    process.stdout.write(summary);
  }

  if (!identical) {
    console.error(
      `Dialect guard failed: legacy ON changed Vue 3 codegen in ${differences.length} file(s).`,
    );
    process.exitCode = 1;
  } else if (regressed) {
    console.error(
      `Dialect guard failed: legacy ON regressed the Vue 3 compile path by ${changePercent.toFixed(2)}% / ${deltaMs.toFixed(1)}ms (threshold ${threshold}% and ${minRegressionMs}ms).`,
    );
    process.exitCode = 1;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
