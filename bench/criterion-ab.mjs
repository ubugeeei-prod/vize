#!/usr/bin/env node
/**
 * Run criterion micro-benchmarks for a base and head checkout and compare the
 * two saved baselines with `critcmp`, surfacing the table in the GitHub Actions
 * step summary.
 *
 * Unlike `compare-pr.mjs` (which times the whole CLI on a generated corpus),
 * this script drives the in-crate criterion suites under the crate `benches`
 * directories. Both sides save a named baseline into the same `--target-dir`,
 * so critcmp can diff `base` against `head` for each benchmark id in one pass.
 *
 * The script is dependency-free (besides `cargo`, `critcmp`, and a checkout of
 * each side) so GitHub Actions can run it after checking out both commits.
 *
 * Cadence note: criterion is noisy on shared CI runners, so this is a reporting
 * gate by default — it prints the critcmp delta table and only fails when
 * `--threshold <pct>` is passed and a benchmark regresses past it. The workflow
 * runs it in report-only mode so micro-benchmark jitter never blocks a PR; the
 * threshold knob is wired so the gate can be tightened later without a code
 * change.
 *
 * Documented JSX regression threshold (#1501): the four JSX cost dimensions —
 * parser/lowering (`jsx_lower`), Croquis semantic analysis
 * (`jsx_croquis_analyze`), Patina rule traversal (`jsx_lint`), and VDOM/Vapor
 * codegen (`jsx_compile_vdom` / `jsx_compile_vapor` / `jsx_compile_mode_aware`) —
 * are all A/B-compared here. When the gate is enabled, run with
 * `--threshold 10`: a +10% median regression on any of these ids fails the run.
 * 10% sits above the run-to-run jitter we observe for these microsecond-scale
 * benches on shared runners (so it does not false-positive) while still catching
 * a real algorithmic regression. Set `CRITERION_AB_THRESHOLD: 10` in
 * `.github/workflows/criterion-bench.yml` to flip the report-only lane into a
 * hard gate without any code change.
 */

import { spawnSync } from "node:child_process";
import { appendFileSync, existsSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

// Criterion benches that exist under crates/*/benches and represent the hot
// compiler/analysis/codegen paths. Each entry maps a cargo package to the
// `[[bench]]` targets it owns; the `bench filter` narrows criterion to the
// specific group so a full sweep stays inside the job timeout.
export const CRITERION_SUITES = [
  {
    package: "vize_atelier_sfc",
    benches: ["sfc_parse", "sfc_compile"],
    label: "SFC parse + compile",
  },
  // jsx_compile owns the JSX parser/lowering, Croquis-analysis
  // (`jsx_croquis_analyze`), and VDOM/Vapor backend dimensions (#1501);
  // markup_ir_bench's `jsx_lint` group covers the Patina rule-traversal cost on
  // JSX. Both targets are A/B-compared so a regression in any of the four JSX
  // cost dimensions surfaces here.
  { package: "vize_atelier_jsx", benches: ["jsx_compile"], label: "JSX compile" },
  { package: "vize_croquis_cf", benches: ["cross_file"], label: "Cross-file analysis" },
  { package: "vize_glyph", benches: ["formatter"], label: "Formatter" },
  { package: "vize_patina", benches: ["lint_bench", "markup_ir_bench"], label: "Lint" },
];

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

function parsePositiveFloat(value) {
  const parsed = Number.parseFloat(value ?? "");
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
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

/**
 * Build the `cargo bench` argument vector for one side of the comparison.
 * `targetDir` is shared between base and head so critcmp can read both
 * baselines; `baseline` is the criterion baseline name (`base` or `head`).
 */
export function cargoBenchArgs({ pkg, benches, baseline, targetDir }) {
  const args = ["bench", "-p", pkg];
  for (const bench of benches) {
    args.push("--bench", bench);
  }
  args.push("--target-dir", targetDir);
  // Everything after `--` is forwarded to the criterion harness.
  args.push("--", "--save-baseline", baseline);
  return args;
}

function benchSide({ side, checkoutDir, baseline, targetDir, suites }) {
  for (const suite of suites) {
    const args = cargoBenchArgs({
      pkg: suite.package,
      benches: suite.benches,
      baseline,
      targetDir,
    });
    console.log(`\n==> [${side}] cargo ${args.join(" ")}`);
    run("cargo", args, { cwd: checkoutDir });
  }
}

function critcmpCompare({ targetDir, threshold }) {
  // `--target-dir` points criterion at <dir>/criterion; critcmp reads the same
  // location via CRITERION_HOME so both baselines resolve without extra flags.
  const env = { CRITERION_HOME: resolve(targetDir, "criterion") };
  const args = ["base", "head"];
  if (threshold != null) {
    // critcmp's own threshold only colorizes; we still parse the table below to
    // decide pass/fail so the behaviour is identical across critcmp versions.
    args.push("--threshold", String(threshold));
  }
  const result = run("critcmp", args, { capture: true, allowFailure: true, env });
  return `${result.stdout ?? ""}${result.stderr ?? ""}`;
}

/**
 * Parse a critcmp table into per-benchmark base/head nanosecond medians.
 * critcmp prints rows like:
 *   group/bench   1.00   3.2±0.1µs   1.04  3.4±0.2µs
 * where the two timing columns are base and head. We only need the ratio, which
 * critcmp already encodes as the leading multiplier on each side (1.00 for the
 * faster side). Rather than re-derive units we read the explicit factor columns.
 */
export function parseCritcmpRegressions(table, thresholdPercent) {
  if (thresholdPercent == null) {
    return [];
  }
  const regressions = [];
  for (const rawLine of table.split("\n")) {
    const line = rawLine.trim();
    if (!line || line.startsWith("group")) {
      continue;
    }
    // Columns: <name> <baseFactor> <baseTime> <headFactor> <headTime>
    const match = line.match(/^(\S+)\s+(\d+(?:\.\d+)?)\s+\S+\s+(\d+(?:\.\d+)?)\s+\S+/);
    if (!match) {
      continue;
    }
    const name = match[1];
    const baseFactor = Number.parseFloat(match[2]);
    const headFactor = Number.parseFloat(match[3]);
    if (!Number.isFinite(baseFactor) || !Number.isFinite(headFactor) || baseFactor === 0) {
      continue;
    }
    // critcmp normalises the faster column to 1.00; head/base ratio is the
    // head factor when base is the 1.00 baseline, otherwise its reciprocal.
    const ratio = headFactor / baseFactor;
    const changePercent = (ratio - 1) * 100;
    if (changePercent >= thresholdPercent) {
      regressions.push({ name, changePercent });
    }
  }
  return regressions;
}

export function renderSummary({ table, threshold, regressions }) {
  const lines = [];
  lines.push("## Criterion A/B");
  lines.push("");
  lines.push(
    threshold == null
      ? "Report-only: micro-benchmark medians for base vs head (no gate)."
      : `Regression threshold: ${threshold}%.`,
  );
  lines.push("");
  lines.push("```");
  lines.push(table.trimEnd());
  lines.push("```");
  if (regressions.length > 0) {
    lines.push("");
    lines.push(`Regressions past ${threshold}%:`);
    for (const regression of regressions) {
      lines.push(`- ${regression.name}: +${regression.changePercent.toFixed(2)}%`);
    }
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const baseDir = resolve(requireArg(args, "base-dir"));
  const headDir = resolve(requireArg(args, "head-dir"));
  const targetDir = resolve(requireArg(args, "target-dir"));
  const threshold = parsePositiveFloat(args.threshold);

  if (!existsSync(baseDir)) {
    throw new Error(`Base checkout not found: ${baseDir}`);
  }
  if (!existsSync(headDir)) {
    throw new Error(`Head checkout not found: ${headDir}`);
  }

  // Base first, then head, into the shared target dir so critcmp sees both.
  benchSide({
    side: "base",
    checkoutDir: baseDir,
    baseline: "base",
    targetDir,
    suites: CRITERION_SUITES,
  });
  benchSide({
    side: "head",
    checkoutDir: headDir,
    baseline: "head",
    targetDir,
    suites: CRITERION_SUITES,
  });

  const table = critcmpCompare({ targetDir, threshold });
  const regressions = parseCritcmpRegressions(table, threshold);
  const summary = renderSummary({ table, threshold, regressions });

  if (args.out) {
    writeFileSync(resolve(args.out), summary);
  }
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
  } else {
    process.stdout.write(summary);
  }

  if (threshold != null && regressions.length > 0) {
    console.error(
      `Criterion budget failed: ${regressions.length} benchmark(s) regressed past ${threshold}%.`,
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
