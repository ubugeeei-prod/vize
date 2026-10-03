import assert from "node:assert/strict";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";
import { isMain, cliArgs, directory, save, sha256 } from "./common.ts";
import { projects } from "./projects.ts";
import { verifyRuntime } from "./preflight.ts";

export function validateReport(report: any, project: (typeof projects)[number], expected: string) {
  assert.equal(report.schema, "vize.fixtureLintDivergenceRun");
  assert.equal(report.version, 2);
  assert.equal(report.project, project.id);
  assert.equal(report.revision, project.revision);
  assert.equal(report.evidence.commitSha, expected);
  assert.equal(report.preset, "ecosystem");
  assert.equal(report.baseline.version, "10.9.2");
  assert.ok(report.files.comparedCount > 0);
  const divergence = report.divergence,
    count = divergence.summary.baselineInvalidRangeCount;
  assert.ok(Number.isSafeInteger(count) && count >= 0);
  if (count) {
    assert.ok(Array.isArray(divergence.baselineInvalidRanges));
    assert.equal(
      divergence.baselineInvalidRanges.length,
      count,
      "Full original finding count differs",
    );
  } else
    assert.equal(
      Object.hasOwn(divergence, "baselineInvalidRanges"),
      false,
      "Valid report shape changed",
    );
  assert.equal(report.budget.maxFalsePositiveCount, 0);
  assert.equal(report.budget.maxFalseNegativeCount, 0);
  if (count) {
    assert.equal(report.budget.verdict, "unusable");
    assert.equal(report.budget.passed, false);
  }
  assert.match(divergence.sha256, /^[0-9a-f]{64}$/u);
  for (const record of divergence.baselineInvalidRanges ?? []) {
    assert.equal(Object.keys(record).sort().join(","), "file,finding");
    assert.ok(
      record.finding && typeof record.finding === "object" && !Array.isArray(record.finding),
    );
    assert.equal(typeof record.finding.ruleId, "string");
    assert.ok(
      typeof record.file === "string" && !isAbsolute(record.file) && record.file.endsWith(".vue"),
    );
    assert.ok(
      !record.file.includes("\\") &&
        !record.file.split("/").some((part: string) => ["", ".", ".."].includes(part)),
    );
  }
  return count;
}
export function validateSummary(summary: any, reports: any[], expected: string) {
  assert.equal(summary.schema, "vize.fixtureLintDivergenceIndex");
  assert.equal(summary.version, 1);
  assert.equal(summary.evidence.commitSha, expected);
  assert.equal(summary.preset, "ecosystem");
  assert.equal(summary.projectCount, 19);
  assert.equal(summary.budget.projectCount, 19);
  assert.deepEqual(
    summary.projects.map((p: any) => p.project).sort(),
    projects.map((p) => p.id).sort(),
  );
  assert.equal(reports.length, 19);
  const count = reports.reduce((sum, report) => sum + report.actualInvalidCount, 0);
  assert.equal(summary.totals.baselineInvalidRangeCount, count);
  return count;
}
export function verifyReports(root: string, expected: string) {
  const preflight = verifyRuntime(root, expected, "post-reporter-reports"),
    out = join(directory(root), "reports");
  const reports = projects.map((project) => {
    const path = join(out, project.id + "-lint-divergence.json"),
      bytes = readFileSync(path),
      report = JSON.parse(bytes.toString());
    const actualInvalidCount = validateReport(report, project, expected);
    const files: string[] = [
      ...new Set<string>((report.divergence.baselineInvalidRanges ?? []).map((r: any) => r.file)),
    ];
    const invalidFiles = files.map((file) => {
      const cwd = realpathSync(join(root, project.fixturePath)),
        absolute = realpathSync(join(cwd, file));
      assert.ok(!relative(cwd, absolute).startsWith("..") && !isAbsolute(relative(cwd, absolute)));
      const sourceBytes = readFileSync(absolute);
      return { file, bytes: sourceBytes.length, sha256: sha256(sourceBytes) };
    });
    return {
      project: project.id,
      path,
      bytes: bytes.length,
      sha256: sha256(bytes),
      producerEvidenceSha256: report.divergence.sha256,
      historicalCount: project.historicalCount,
      actualInvalidCount,
      verdict: report.budget.verdict,
      invalidFiles,
    };
  });
  const summaryPath = join(out, "lint-divergence-summary.json"),
    summaryBytes = readFileSync(summaryPath);
  const freshInvalidCount = validateSummary(JSON.parse(summaryBytes.toString()), reports, expected);
  return {
    ...preflight,
    mode: "reports",
    reports,
    freshInvalidCount,
    freshCountRequiredToEqualHistorical: false,
    summary: { path: summaryPath, bytes: summaryBytes.length, sha256: sha256(summaryBytes) },
    hashQualification:
      "Complete raw project bytes and canonical Rust hashes retained; no replacement or recomputation of producer hashes.",
    acceptance: false,
  };
}
if (isMain(import.meta.url)) {
  const { mode, root, expected } = cliArgs();
  assert.equal(mode, "reports");
  assert.ok(!existsSync(join(directory(root), "report-verification.json")));
  try {
    save(root, "report-verification.json", { complete: true, ...verifyReports(root, expected) });
  } catch (error) {
    save(root, "report-verification.json", {
      complete: false,
      expectedSource: expected,
      acceptance: false,
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}
