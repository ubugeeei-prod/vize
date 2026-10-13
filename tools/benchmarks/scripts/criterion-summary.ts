/**
 * Render the Criterion A/B step summary: suite selection, the report-only
 * critcmp table, relative regressions when a threshold is enabled, and the
 * absolute median budgets that stay hard gates on the reference runner.
 */

import type {
  AbsoluteBudgetResult,
  CriterionRegression,
  VaporPairResult,
} from "./criterion-baselines.ts";
import { isRecord, isStringArray } from "./script-types.ts";

/** Which configured suites ran, which were skipped, and why. */
export type SuiteSelection = { selected: string[]; skipped: string[]; reason: string };

/** An impact manifest from `criterion-impact.ts`: the suites to run and why. */
export function isSuiteManifest(
  value: unknown,
): value is Pick<SuiteSelection, "selected" | "reason"> {
  return isRecord(value) && isStringArray(value.selected) && typeof value.reason === "string";
}

export function renderSummary({
  table,
  threshold,
  regressions,
  selection,
  absoluteBudgetResults = [],
  vaporPairResults = [],
}: {
  table: string;
  threshold: number | undefined;
  regressions: CriterionRegression[];
  selection: SuiteSelection;
  absoluteBudgetResults?: AbsoluteBudgetResult[];
  vaporPairResults?: VaporPairResult[];
}): string {
  const lines: string[] = [];
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
      ? "Relative comparison: micro-benchmark estimates for base vs head (report-only)."
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
  if (absoluteBudgetResults.length > 0) {
    lines.push("");
    lines.push("### Absolute median budgets");
    lines.push("");
    lines.push("| Benchmark | Median | Budget | Result |");
    lines.push("| --- | ---: | ---: | :---: |");
    for (const result of absoluteBudgetResults) {
      lines.push(
        `| ${result.name} | ${formatDuration(result.medianNs)} | ${formatDuration(result.maxMedianNs)} | ${result.exceeded ? "FAIL" : "PASS"} |`,
      );
    }
  }
  if (vaporPairResults.length > 0) {
    lines.push("");
    lines.push("### Davinci Vapor native vs retained (report-only)");
    lines.push("");
    lines.push(
      "Each ratio is L3 native median / explicitly retained median on the same source and options; below 1.000x is faster. Base and head use the same Blacksmith runner, with isolated Cargo build graphs. Criterion executes cases sequentially, so these are observations rather than a paired-interleaved throughput gate.",
    );
    lines.push("");
    lines.push(
      "| Fixture | Base native | Base retained | Base ratio | Head native | Head retained | Head ratio |",
    );
    lines.push("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (const result of vaporPairResults) {
      lines.push(
        `| ${result.fixture} | ${formatDuration(result.baseNativeNs)} | ${formatDuration(result.baseRetainedNs)} | ${result.baseRatio.toFixed(3)}x | ${formatDuration(result.headNativeNs)} | ${formatDuration(result.headRetainedNs)} | ${result.headRatio.toFixed(3)}x |`,
      );
    }
    lines.push("");
    lines.push(
      "Some native and retained fixtures (for example object spreads) emit different programs; those ratios are diagnostic only, not a parity or promotion verdict.",
    );
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

function formatDuration(nanoseconds: number): string {
  if (nanoseconds >= 1_000_000) {
    return `${(nanoseconds / 1_000_000).toFixed(2)} ms`;
  }
  if (nanoseconds >= 1_000) {
    return `${(nanoseconds / 1_000).toFixed(2)} µs`;
  }
  return `${nanoseconds.toFixed(2)} ns`;
}
