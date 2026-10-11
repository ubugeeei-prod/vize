import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

import { validateTypecheckerOutput } from "../../tools/support/compat/fixtures/tool-matrix-typechecker.mjs";
import { compareTypecheckDiagnostics } from "../../tools/support/compat/fixtures/typecheck-divergence.mjs";

const evidence = new URL("./fixtures/typechecker-native-hints/", import.meta.url);
const receipts = JSON.parse(readFileSync(new URL("receipts.json", evidence), "utf8"));

for (const receipt of receipts.reports) {
  test(`original ${receipt.project} report keeps every native hint and whole JSON byte`, () => {
    const raw = gunzipSync(readFileSync(new URL(`${receipt.project}.json.gz`, evidence)));
    assert.equal(raw.length, receipt.stdoutBytes);
    assert.equal(createHash("sha256").update(raw).digest("hex"), receipt.stdoutSha256);
    const report = JSON.parse(raw.toString("utf8"));
    const original = structuredClone(report);
    assert.equal(report.errorCount, receipt.errorCount);
    assert.equal(report.warningCount, receipt.warningCount);
    assert.equal(report.fileCount, receipt.fileCount);
    const checked = report.files.slice(0, report.fileCount).map((file: any) => file.file);
    const requested = checked.filter((file: string) => file.endsWith(".vue"));
    const coverage = validateTypecheckerOutput(
      { id: receipt.project },
      report,
      receipt.exitCode,
      requested,
      checked,
    );
    assert.equal(coverage.checked.fileCount, receipt.fileCount);
    assert.equal(coverage.requested.fileCount, requested.length);
    assert.equal(
      report.files
        .flatMap((file: any) => file.diagnostics)
        .filter((diagnostic: string) => diagnostic.startsWith("hint:")).length,
      receipt.hintCount,
    );
    assert.deepEqual(report, original);
    assert.equal(JSON.stringify(report), JSON.stringify(original));
  });
}

function hintReport(diagnostic = "hint:77:22 [TS6385] 'typescript' is deprecated.") {
  return {
    files: [{ file: "src/App.vue", diagnostics: [diagnostic] }],
    errorCount: 0,
    warningCount: 0,
    fileCount: 1,
  };
}

test("hint-only reports retain zero error/warning counts and successful process status", () => {
  const report = hintReport();
  const original = structuredClone(report);
  validateTypecheckerOutput({ id: "hints" }, report, 0);
  assert.deepEqual(report, original);
  for (const [field, value] of [
    ["errorCount", 1],
    ["warningCount", 1],
  ] as const) {
    assert.throws(
      () => validateTypecheckerOutput({ id: "hints" }, { ...report, [field]: value }, 0),
      /does not match 0 diagnostics/,
    );
  }
  assert.throws(() => validateTypecheckerOutput({ id: "hints" }, report, 1), /exit code/);
});

test("hint severity remains a full divergence finding against the unchanged baseline", () => {
  const report = hintReport();
  const comparison = compareTypecheckDiagnostics({
    projectId: "hints",
    cwd: "/fixture",
    vizeReport: report,
    vueTscOutput: "",
  });
  assert.equal(comparison.summary.vizeDiagnosticCount, 1);
  assert.equal(comparison.summary.vizeHintCount, 1);
  assert.equal(comparison.summary.falsePositiveCount, 1);
  assert.equal(comparison.summary.falsePositiveRatio, 1);
  assert.deepEqual(comparison.falsePositives, [
    {
      file: "src/App.vue",
      severity: "hint",
      line: 77,
      column: 22,
      code: 6385,
      message: "'typescript' is deprecated.",
    },
  ]);
  const mismatch = compareTypecheckDiagnostics({
    projectId: "hints",
    cwd: "/fixture",
    vizeReport: report,
    vueTscOutput: "src/App.vue(77,22): warning TS6385: 'typescript' is deprecated.\n",
  });
  assert.equal(mismatch.summary.sharedCount, 0);
  assert.equal(mismatch.summary.falsePositiveCount, 1);
  assert.equal(mismatch.summary.falseNegativeCount, 1);
});

test("malformed native hint severity, location, code and message remain fatal", () => {
  for (const diagnostic of [
    "info:77:22 [TS6385] deprecated",
    "hint: malformed",
    "hint:0:22 [TS6385] deprecated",
    "hint:77:0 [TS6385] deprecated",
    "hint:-1:22 [TS6385] deprecated",
    "hint:1.5:22 [TS6385] deprecated",
    "hint:9007199254740992:22 [TS6385] deprecated",
    "hint:77:9007199254740992 [TS6385] deprecated",
    "hint:77:22 [TS0] deprecated",
    "hint:77:22 [TS9007199254740992] deprecated",
    "hint:77:22 [TS6385]",
    "hint:77:22 [TS6385]   ",
  ]) {
    const report = hintReport(diagnostic);
    assert.throws(() => validateTypecheckerOutput({ id: "hints" }, report, 0));
    assert.throws(() =>
      compareTypecheckDiagnostics({
        projectId: "hints",
        cwd: "/fixture",
        vizeReport: report,
        vueTscOutput: "",
      }),
    );
  }
});

test("canonical Rust validators execute the same native hint contracts", () => {
  for (const name of ["tool-matrix-report", "typecheck-divergence-report"]) {
    const tool = new URL(`../../tools/commands/fixtures/${name}.rs`, import.meta.url);
    const result = spawnSync("rust-script", ["--test", fileURLToPath(tool)], {
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr + result.stdout);
  }
});
