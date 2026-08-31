/**
 * Render the Criterion A/B step summary: suite selection, the report-only
 * critcmp table, relative regressions when a threshold is enabled, and the
 * absolute median budgets that stay hard gates on the reference runner.
 */

export function renderSummary({
  table,
  threshold,
  regressions,
  selection,
  absoluteBudgetResults = [],
}) {
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
  lines.push("");
  return `${lines.join("\n")}\n`;
}

function formatDuration(nanoseconds) {
  if (nanoseconds >= 1_000_000) {
    return `${(nanoseconds / 1_000_000).toFixed(2)} ms`;
  }
  if (nanoseconds >= 1_000) {
    return `${(nanoseconds / 1_000).toFixed(2)} µs`;
  }
  return `${nanoseconds.toFixed(2)} ns`;
}
