#!/usr/bin/env node
/**
 * Run criterion micro-benchmarks for a base and head checkout and compare the
 * two saved baselines with `critcmp`, surfacing the table in the GitHub Actions
 * step summary.
 *
 * Unlike `compare-pr.mjs` (which times the whole CLI on a generated corpus),
 * this script drives the in-crate criterion suites under the crate `benches`
 * directories. Each checkout builds in an isolated Cargo target directory, then
 * critcmp compares exported JSON snapshots for each benchmark id in one pass.
 *
 * The script is dependency-free (besides `cargo`, `critcmp`, and a checkout of
 * each side) so GitHub Actions can run it after checking out both commits.
 *
 * Cadence note: relative Criterion deltas are noisy on shared CI runners, so
 * the global percentage comparison is report-only by default. Suites can also
 * declare conservative absolute median budgets for the reference runner; those
 * remain hard gates even when no global `--threshold <pct>` is supplied.
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
import { appendFileSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import {
  compareBaselineExports,
  criterionEnvironment,
  critcmpArgs,
  critcmpExportArgs,
  evaluateAbsoluteBudgets,
  parseCritcmpExport,
  validateComparisonTable,
} from "./criterion-baselines.mjs";
import { renderSummary } from "./criterion-summary.mjs";

// Criterion benches that exist in workspace benchmark targets and represent
// hot compiler, analysis, codegen, and presentation paths. Each entry maps a
// cargo package to the `[[bench]]` targets it owns; the `bench filter` narrows
// Criterion to the specific group so a full sweep stays inside the job timeout.
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
  { package: "vize_doctor", benches: ["reporter"], label: "Doctor reporters" },
  {
    package: "vize_benchmarks",
    benches: ["doctor_tui"],
    label: "Doctor TUI",
    absoluteBudgets: [
      { name: "doctor_tui_10k/first_frame_120x40", maxMedianNs: 20_000_000 },
      { name: "doctor_tui_input_to_frame_10k/selection", maxMedianNs: 1_000_000 },
      { name: "doctor_tui_input_to_frame_10k/search", maxMedianNs: 1_000_000 },
    ],
  },
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
  if (result.status !== 0) {
    const output = `${result.stdout ?? ""}${result.stderr ?? ""}`.trim();
    const details = output ? `\n${output}` : "";
    throw new Error(`${command} ${commandArgs.join(" ")} exited with ${result.status}${details}`);
  }
  return result;
}

/**
 * Build the `cargo bench` argument vector for one side of the comparison.
 * `targetDir` is side-specific so Cargo never reuses base checkout crate
 * metadata while compiling the head checkout; `baseline` is the criterion
 * baseline name (`base` or `head`).
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

export function criterionBenchRunOptions({ checkoutDir, targetDir }) {
  return {
    cwd: checkoutDir,
    env: criterionEnvironment(targetDir),
    capture: false,
  };
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
    run("cargo", args, criterionBenchRunOptions({ checkoutDir, targetDir }));
  }
}

function exportBaseline({ targetDir, baseline, outputPath }) {
  const result = run("critcmp", critcmpExportArgs({ targetDir, baseline }), { capture: true });
  const serialized = result.stdout ?? "";
  const parsed = parseCritcmpExport(serialized, baseline);
  writeFileSync(outputPath, serialized);
  return parsed;
}

function critcmpCompare({ targetDir, baselinePaths }) {
  // critcmp 0.1.8 requires a target-dir argument even when explicit exported
  // snapshot paths are supplied. The head target is enough for its scratch data.
  const args = critcmpArgs({ targetDir, baselinePaths });
  const result = run("critcmp", args, { capture: true });
  return `${result.stdout ?? ""}${result.stderr ?? ""}`;
}

export function criterionSideTargetDirs(targetDir) {
  return {
    baseTargetDir: resolve(targetDir, "base-target"),
    headTargetDir: resolve(targetDir, "head-target"),
  };
}

export function resolveSuiteSelection(selection) {
  const inventory = CRITERION_SUITES.map((suite) => suite.package);
  if (selection == null) {
    return {
      selected: inventory,
      skipped: [],
      reason: "Full inventory selected (no impact manifest supplied).",
    };
  }
  if (!Array.isArray(selection.selected) || typeof selection.reason !== "string") {
    throw new Error("Criterion impact manifest must contain selected[] and reason");
  }
  if (new Set(selection.selected).size !== selection.selected.length) {
    throw new Error("Criterion impact manifest contains duplicate suites");
  }
  const unknown = selection.selected.filter((name) => !inventory.includes(name));
  if (unknown.length > 0) {
    throw new Error(`Criterion impact manifest contains unknown suites: ${unknown.join(", ")}`);
  }
  return {
    selected: inventory.filter((name) => selection.selected.includes(name)),
    skipped: inventory.filter((name) => !selection.selected.includes(name)),
    reason: selection.reason,
  };
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const baseDir = resolve(requireArg(args, "base-dir"));
  const headDir = resolve(requireArg(args, "head-dir"));
  const targetDir = resolve(requireArg(args, "target-dir"));
  const threshold = parsePositiveFloat(args.threshold);
  const selection = resolveSuiteSelection(
    args.selection ? JSON.parse(readFileSync(resolve(args.selection), "utf8")) : undefined,
  );

  if (!existsSync(baseDir)) {
    throw new Error(`Base checkout not found: ${baseDir}`);
  }
  if (!existsSync(headDir)) {
    throw new Error(`Head checkout not found: ${headDir}`);
  }

  const suites = CRITERION_SUITES.filter((suite) => selection.selected.includes(suite.package));
  const basePath = resolve(targetDir, "criterion-ab-base.json");
  const headPath = resolve(targetDir, "criterion-ab-head.json");
  const { baseTargetDir, headTargetDir } = criterionSideTargetDirs(targetDir);
  let baseExport;
  let headExport;
  // Export each side from its own target dir, then compare immutable snapshots.
  if (suites.length > 0) {
    benchSide({
      side: "base",
      checkoutDir: baseDir,
      baseline: "base",
      targetDir: baseTargetDir,
      suites,
    });
    baseExport = exportBaseline({
      targetDir: baseTargetDir,
      baseline: "base",
      outputPath: basePath,
    });
    benchSide({
      side: "head",
      checkoutDir: headDir,
      baseline: "head",
      targetDir: headTargetDir,
      suites,
    });
    headExport = exportBaseline({
      targetDir: headTargetDir,
      baseline: "head",
      outputPath: headPath,
    });
  }

  const table =
    suites.length > 0
      ? critcmpCompare({ targetDir: headTargetDir, baselinePaths: [basePath, headPath] })
      : "";
  if (suites.length > 0) {
    validateComparisonTable(table);
  }
  const regressions =
    baseExport && headExport ? compareBaselineExports(baseExport, headExport, threshold) : [];
  const absoluteBudgets = suites.flatMap((suite) => suite.absoluteBudgets ?? []);
  const absoluteBudgetResults =
    headExport && absoluteBudgets.length > 0
      ? evaluateAbsoluteBudgets(headExport, absoluteBudgets)
      : [];
  const summary = renderSummary({
    table,
    threshold,
    regressions,
    selection,
    absoluteBudgetResults,
  });

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
  const exceededBudgets = absoluteBudgetResults.filter((result) => result.exceeded);
  if (exceededBudgets.length > 0) {
    console.error(
      `Criterion absolute budget failed: ${exceededBudgets.length} benchmark(s) exceeded their median limit.`,
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
