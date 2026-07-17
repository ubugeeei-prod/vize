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
import { appendFileSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import {
  compareBaselineExports,
  criterionEnvironment,
  critcmpArgs,
  critcmpExportArgs,
  parseCritcmpExport,
  validateComparisonTable,
} from "./criterion-baselines.mjs";

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
  if (result.status !== 0) {
    const output = `${result.stdout ?? ""}${result.stderr ?? ""}`.trim();
    const details = output ? `\n${output}` : "";
    throw new Error(`${command} ${commandArgs.join(" ")} exited with ${result.status}${details}`);
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
    run("cargo", args, {
      cwd: checkoutDir,
      env: criterionEnvironment(targetDir),
    });
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
  // critcmp 0.1.8 does not read CRITERION_HOME. Point it at the same Cargo
  // target directory used by both benchmark runs. The exported snapshots keep
  // the base sample stable while the head checkout reuses the shared target.
  const args = critcmpArgs({ targetDir, baselinePaths });
  const result = run("critcmp", args, { capture: true });
  return `${result.stdout ?? ""}${result.stderr ?? ""}`;
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

export function renderSummary({ table, threshold, regressions, selection }) {
  const lines = [];
  lines.push("## Criterion A/B");
  lines.push("");
  lines.push(`Selection: ${selection.reason}`);
  lines.push("");
  lines.push(`- Ran: ${selection.selected.join(", ") || "none"}`);
  lines.push(`- Skipped: ${selection.skipped.join(", ") || "none"}`);
  lines.push("");
  if (selection.selected.length === 0) {
    lines.push("No configured Criterion suite is affected; timing execution was skipped.");
    lines.push("");
    return `${lines.join("\n")}\n`;
  }
  lines.push(
    threshold == null
      ? "Report-only: micro-benchmark mean estimates for base vs head (no gate)."
      : `Regression threshold: ${threshold}% (median).`,
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
  let baseExport;
  let headExport;
  // Export base before the head run can mutate the shared Criterion directory.
  if (suites.length > 0) {
    benchSide({
      side: "base",
      checkoutDir: baseDir,
      baseline: "base",
      targetDir,
      suites,
    });
    baseExport = exportBaseline({ targetDir, baseline: "base", outputPath: basePath });
    benchSide({
      side: "head",
      checkoutDir: headDir,
      baseline: "head",
      targetDir,
      suites,
    });
    headExport = exportBaseline({ targetDir, baseline: "head", outputPath: headPath });
  }

  const table =
    suites.length > 0 ? critcmpCompare({ targetDir, baselinePaths: [basePath, headPath] }) : "";
  if (suites.length > 0) {
    validateComparisonTable(table);
  }
  const regressions =
    baseExport && headExport ? compareBaselineExports(baseExport, headExport, threshold) : [];
  const summary = renderSummary({ table, threshold, regressions, selection });

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
