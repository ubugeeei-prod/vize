import assert from "node:assert/strict";

// Project the preserved hand-authored service oracle, never observed output.
export function position(source, offset) {
  const lines = Buffer.from(source).subarray(0, offset).toString().split("\n");
  return { line: lines.length, column: lines.at(-1).length + 1, offset };
}
export function nativeExpected(row) {
  return {
    filename: row.filename,
    errorCount: row.service.error_count,
    warningCount: row.service.warning_count,
    diagnostics: row.service.diagnostics.map((d) => ({
      rule: d.rule_name,
      severity: d.severity,
      message: d.message,
      location: { start: position(row.source, d.start), end: position(row.source, d.end) },
      help: d.help,
    })),
  };
}
export function cliExpected(row) {
  const native = nativeExpected(row);
  return [
    {
      file: row.filename,
      messages: native.diagnostics.map((d) => ({
        ruleId: d.rule,
        ruleDocsPath: `docs/content/rules/${d.rule.startsWith("ssr/") ? "ssr" : "vue"}.md`,
        severity: d.severity === "error" ? 2 : 1,
        message: `[vize:${d.rule}] ${d.message}`,
        line: d.location.start.line,
        column: d.location.start.column,
        endLine: d.location.end.line,
        endColumn: d.location.end.column,
        ...(d.help === null ? {} : { help: d.help }),
      })),
      errorCount: native.errorCount,
      warningCount: native.warningCount,
    },
  ];
}
export function pluginExpected(row, originalLocations) {
  return nativeExpected(row).diagnostics.map((d) => {
    const inScript = d.location.end.offset <= row.source.indexOf("</script>");
    const mapped = originalLocations || inScript;
    const start = mapped ? d.location.start : position(row.source, row.source.indexOf(">") + 2);
    return {
      message: mapped
        ? d.message
        : `${d.message} (at <art>:${d.location.start.line}:${d.location.start.column})`,
      code: `vize(${d.rule})`,
      severity: d.severity,
      causes: [],
      filename: row.filename,
      labels: [
        {
          span: {
            offset: start.offset,
            length: mapped ? d.location.end.offset - start.offset : 0,
            line: start.line,
            column: start.column,
          },
        },
      ],
      related: [],
    };
  });
}
export function checkPluginReport(report, expected) {
  assert.deepEqual(Object.keys(report).sort(), [
    "diagnostics",
    "number_of_files",
    "number_of_rules",
    "start_time",
    "threads_count",
  ]);
  assert.equal(report.number_of_files, 1);
  assert.ok(Number.isSafeInteger(report.number_of_rules) && report.number_of_rules > 0);
  assert.ok(Number.isSafeInteger(report.threads_count) && report.threads_count >= 0);
  assert.ok(Number.isFinite(report.start_time) && report.start_time >= 0);
  assert.deepEqual(report.diagnostics, expected);
}
